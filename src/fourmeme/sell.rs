use ethers::prelude::*;
use ethers::types::{U256, Address, transaction::eip2718::TypedTransaction};
use std::sync::Arc;
use anyhow::Result;

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
pub async fn get_sell_token_tx<M: Middleware>(
    client: Arc<SignerMiddleware<M, LocalWallet>>,
    token_manager_address: Address,
    token_address: Address,      // ✅ token contract address
    sell_amount_tokens: f64,     // ✅ human-readable token amount (e.g. 25.0)
) -> Result<TypedTransaction> {

    // Connect to TokenManager2 contract
    let token_manager = TokenManager2::new(token_manager_address, client);

    // Convert token amount (human-readable) to wei (18 decimals)
    let sell_amount_wei = U256::from((sell_amount_tokens * 1e18_f64) as u128);

    // Build the `sellToken(address,uint256)` call
    let call = token_manager.sell_token(token_address, sell_amount_wei);

    // Return as TypedTransaction for signing/bundling
    Ok(call.tx)
}
