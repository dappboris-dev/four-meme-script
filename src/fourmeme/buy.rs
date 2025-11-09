use ethers::prelude::*;
use ethers::types::{U256, Address, transaction::eip2718::TypedTransaction};
use std::sync::Arc;
use anyhow::{Result, Context};
abigen!(
    TokenManager2,
    r#"[ 
        function buyToken(address token, uint256 amount, uint256 maxFunds) payable
        function buyTokenAMAP(address token, uint256 funds, uint256 minAmount) payable
        function sellToken(address token, uint256 amount)
        function sellTokenAMAP(address token, uint256 amount, uint256 minBNB)
    ]"#
);

abigen!(
    ERC20,
    r#"[ 
        function approve(address spender, uint256 amount) returns (bool)
    ]"#
);

abigen!(
    TokenManagerHelper3,
    r#"
    [
        function tryBuy(address token, uint256 amount, uint256 funds) view returns (address tokenManager, address quote, uint256 estimatedAmount, uint256 estimatedCost, uint256 estimatedFee, uint256 amountMsgValue, uint256 amountApproval, uint256 amountFunds)
        function trySell(address token, uint256 amount) view returns (address tokenManager, address quote, uint256 funds, uint256 fee)
    ]
    "#
);
pub async fn get_buy_token_tx(
    wallet: LocalWallet,
    provider: Arc<Provider<Http>>,
    token_manager_address: Address,
    token_address: Address,
    token_amount: f64,       // exact number of tokens
    max_bnb_ether: &str,     // max BNB to spend, e.g., "0.2"
) -> Result<TypedTransaction> {
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    // Connect contract
    let contract = TokenManager2::new(token_manager_address, client.clone());

    // Convert amounts to wei
    let token_amount_wei = U256::from((token_amount * 1e18_f64) as u128);
    let max_funds_wei = ethers::utils::parse_ether(max_bnb_ether)?;

    // Build the transaction (unsigned)
    let call = contract
        .buy_token(token_address, token_amount_wei, max_funds_wei)
        .value(max_funds_wei);

    Ok(call.tx)
}

/// Returns a TypedTransaction for buying a token with fixed BNB (AMAP)
pub async fn get_buy_amap_tx(
    wallet: LocalWallet,
    provider: Arc<Provider<Http>>,
    token_manager_address: Address,
    token_address: Address,
    bnb_amount_ether: &str,  // BNB amount to spend
) -> Result<TypedTransaction> {
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    let contract = TokenManager2::new(token_manager_address, client.clone());

    let funds_wei = ethers::utils::parse_ether(bnb_amount_ether)?;

    let call = contract
        .buy_token_amap(token_address, funds_wei, U256::zero())
        .value(funds_wei);

    Ok(call.tx)
}

/// Approves TokenManager2 to spend your ERC20 tokens
pub async fn approve_token(
    wallet: LocalWallet,
    provider: Arc<Provider<Http>>,
    token_address: Address,
    manager_address: Address,
) -> Result<()> {
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    // After (fix)
    let token_contract = ERC20::new(token_address, client.clone());
    let approve_call = token_contract.approve(manager_address, U256::MAX);
    approve_call.send().await?;
    Ok(())
}
