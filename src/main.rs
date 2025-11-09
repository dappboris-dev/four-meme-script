use test_blox::utils::init_client;
use ethers::providers::{Provider, Http};
use ethers::providers::Middleware;
use test_blox::types::WalletInfo;
use ethers::prelude::*;
use test_blox::utils::{gen_several_wallets, distribute_bnb};
use test_blox::bundle::{bundle_bsc_tx};
use test_blox::fourmeme::{get_create_new_token_tx, approve_token, get_buy_token_tx};
use std::env;
use std::{io, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = Provider::<Http>::try_from("https://bsc-dataseed.binance.org")?;
    let provider = Arc::new(provider);
    let client = init_client()?;
    let bn = client.get_block_number().await?;
    dotenv::dotenv().ok(); // load .env variables

    // Get the number of wallets to generate
    let wallet_num: usize = env::var("WALLET_NUM")
        .unwrap_or_else(|_| "5".to_string()) // default 5 wallets
        .parse()
        .expect("WALLET_NUM must be a number");
    println!("Block number: {}", bn);
    // Load wallet number and min/max BNB from .env
    let min_bnb: f64 = env::var("MIN_BNB")?.parse()?;
    let max_bnb: f64 = env::var("MAX_BNB")?.parse()?;
    let token_manager_address: Address = env::var("TOKEN_MANAGER2")?.parse()?;
    let token_address: Address = env::var("TOKEN_ADDRESS")?.parse()?;
    loop {
        println!("1) Distribute BNB to wallets");       // description: funding wallets
        println!("2) Bundle create and buy tokens");              // description: purchase tokens in batch
        println!("3) Sell tokens and sweep back");     // description: sell & consolidate
        println!("4) Exit"); 
        print!("> ");
        use std::io::Write;
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                println!("🚀 Distributing BNB...");
                let amount_per_wallet = 0.01; // Example
            }
            "2" => {
                println!("🟢 Bundle buying tokens...");
                let private_key = env::var("PRIVATE_KEY")?;

                // Parse into a LocalWallet and set chain id (BSC mainnet = 56)
                let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(56u64);

                let bloxroute_auth = std::env::var("BLOXROUTE_AUTH_HEADER")?;
                let token_create_tx = get_create_new_token_tx(wallet.clone(), provider.clone(), "assets/image.png", "assets/config.json").await?;
                let token_create_signed_tx = wallet.sign_transaction(&token_create_tx).await?.to_string();

                // 1️⃣ Approve token first (if needed)
                approve_token(wallet.clone(), provider.clone(), token_address, token_manager_address).await?;
                println!("✅ Approved TokenManager2 to spend your tokens");

                // 2️⃣ Build a buyToken transaction (buy exact token amount)
                let token_amount = 10.0; // 10 tokens
                let max_bnb = "0.2";     // max 0.2 BNB
                let buy_tx = get_buy_token_tx(wallet.clone(), provider.clone(), token_manager_address, token_address, token_amount, max_bnb).await?;
                let token_buy_signed_tx = wallet.sign_transaction(&buy_tx).await?.to_string();
                let signed_txs = vec![
                    token_create_signed_tx,
                    token_buy_signed_tx
                ];
                bundle_bsc_tx(signed_txs, provider.clone(), &bloxroute_auth).await?;
            }
            "3" => {
                println!("🔵 Selling tokens and sweeping...");
            }
            "4" => {
                println!("👋 Exiting...");
                break;
            }
            _ => println!("❌ Invalid option, choose 1-4."),
        }
    }
    Ok(())
}
