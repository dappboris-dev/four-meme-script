// src/client.rs (or wherever you like)
use anyhow::{Context, Result};
use dotenvy::dotenv;
use std::time::{SystemTime, UNIX_EPOCH};
use ethers::prelude::*;
use std::{fs, path::Path};
use rand::Rng;
use std::{env, sync::Arc, time::Duration};
use std::str::FromStr;
use serde_json::from_str;
use crate::types::WalletInfo;

pub fn init_client() -> Result<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>> {
    dotenv().ok();
    let rpc_url = env::var("RPC_URL")
        .unwrap_or_else(|_| "https://bsc-dataseed.binance.org".to_string());
    let private_key = env::var("PRIVATE_KEY")
        .context("Missing PRIVATE_KEY in environment (.env)")?;
    let chain_id: u64 = env::var("CHAIN_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(56u64);
    let provider = Provider::<Http>::try_from(rpc_url.as_str())
        .context("Failed to create HTTP provider from RPC_URL")?
        .interval(Duration::from_millis(300));
    let wallet: LocalWallet = private_key
        .parse::<LocalWallet>()
        .context("Failed to parse PRIVATE_KEY as LocalWallet")?
        .with_chain_id(chain_id);
    let client = SignerMiddleware::new(provider, wallet);
    Ok(Arc::new(client))
}


pub fn gen_several_wallets(wallet_num: usize) -> Result<Vec<WalletInfo>> {
    let mut wallet_list = Vec::new();

    for _ in 0..wallet_num {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        wallet_list.push(WalletInfo {
            address: wallet.address(),
            private_key: hex::encode(wallet.signer().to_bytes()),
        });
    }

    // ✅ Save to timestamped JSON file
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let folder_path = format!("src/wallets");
    fs::create_dir_all(&folder_path)?;
    let file_path = format!("{}/{}.json", folder_path, timestamp);

    let json_data = serde_json::to_string_pretty(&wallet_list)?;
    fs::write(&file_path, json_data)?;

    println!("✅ Wallets saved to: {}", file_path);

    Ok(wallet_list)
}

pub async fn distribute_bnb(
    main_client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    wallet_list: Vec<WalletInfo>,
    min_amount: f64,
    max_amount: f64,
) -> Result<()> {
    let provider = main_client.provider();
    let balance = provider.get_balance(main_client.address(), None).await?;
    println!("💰 Main wallet balance: {} wei", balance);

    let mut rng = rand::thread_rng();

    for w in wallet_list {
        // Random amount between min and max
        let amount_bnb = rng.gen_range(min_amount..=max_amount);
        let amount_wei = ethers::utils::parse_ether(amount_bnb)?;

        let tx = TransactionRequest::new()
            .to(w.address)
            .value(amount_wei);

        let pending = main_client.send_transaction(tx, None).await?;
        println!(
            "✅ Sent {:.6} BNB to {:?} | tx: {:?}",
            amount_bnb, w.address, pending.tx_hash()
        );
    }

    Ok(())
}

/// Reads all JSON wallet files from `src/wallets` folder
pub fn read_all_wallets() -> Result<Vec<WalletInfo>> {
    let mut all_wallets = Vec::new();
    let folder_path = "src/wallets";

    // Check if the folder exists
    if !Path::new(folder_path).exists() {
        println!("⚠️ Folder {} does not exist.", folder_path);
        return Ok(all_wallets);
    }

    // Iterate over all files in the folder
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    let content = fs::read_to_string(&path)?;
                    let wallets: Vec<WalletInfo> = from_str(&content)?;
                    all_wallets.extend(wallets);
                }
            }
        }
    }

    println!("✅ Loaded {} wallets from {}", all_wallets.len(), folder_path);
    Ok(all_wallets)
}
pub async fn sweep(
    main_client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    wallet_list: Vec<WalletInfo>,
) -> Result<()> {
    let provider = main_client.provider();
    let main_address = main_client.address();

    for w in wallet_list {
        // Create a wallet instance from private key
        let wallet = w.private_key.parse::<LocalWallet>()?.with_chain_id(56u64); // BSC
        let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

        // Get balance of current wallet
        let balance = provider.get_balance(client.address(), None).await?;
        if balance.is_zero() {
            println!("⚠️ Wallet {:?} is empty, skipping.", client.address());
            continue;
        }

        // Estimate gas fee
        let gas_price = provider.get_gas_price().await?;
        let gas_limit = 21_000u64; // simple BNB transfer
        let fee = gas_price * gas_limit;

        if balance <= fee {
            println!("⚠️ Wallet {:?} balance too low to cover gas.", client.address());
            continue;
        }

        let sweep_amount = balance - fee;
        let tx = TransactionRequest::pay(main_address, sweep_amount);

        let pending_tx = client.send_transaction(tx, None).await?;
        println!(
            "✅ Sweeping {:?} → {} | tx: {:?} (amount: {} wei)",
            client.address(),
            main_address,
            pending_tx.tx_hash(),
            sweep_amount
        );
    }

    Ok(())
}