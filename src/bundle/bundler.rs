use ethers::prelude::*;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use chrono::Utc;

/// Sends a bundle of signed transactions to BloxRoute on BSC
pub async fn bundle_bsc_tx(
    signed_txs: Vec<String>, // hex strings of signed txs
    provider: Arc<Provider<Http>>,
    bloxroute_auth: &str,
) -> anyhow::Result<Option<String>> {
    // Get current block
    let block_number = provider.get_block_number().await?;
    let target_block = format!("0x{:x}", block_number.as_u64() + 1);

    // Prepare BloxRoute bundle payload
    let bundle_params = json!({
        "transaction": signed_txs,
        "blockchain_network": "BSC-Mainnet",
        "block_number": target_block,
        "mev_builders": { "all": "" }
    });

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "blxr_submit_bundle",
        "params": bundle_params,
        "id": chrono::Utc::now().timestamp()
    });

    // Send request
    let client = Client::new();
    let res = client
        .post("https://api.blxrbdn.com")
        .header("Content-Type", "application/json")
        .header("Authorization", bloxroute_auth)
        .json(&payload)
        .send()
        .await?;

    let status = res.status();
    let text = res.text().await?;
    println!("BloxRoute response (status {}): {}", status, text);

    // Parse response JSON
    let resp_json: serde_json::Value = serde_json::from_str(&text)?;
    if status.is_success() {
        if let Some(bundle_hash) = resp_json["result"]["bundleHash"].as_str() {
            println!("✅ Bundle accepted! Hash: {}", bundle_hash);
            return Ok(Some(bundle_hash.to_string()));
        } else if let Some(result_str) = resp_json["result"].as_str() {
            return Ok(Some(result_str.to_string()));
        }
    } else if let Some(error) = resp_json["error"].as_object() {
        println!("❌ BloxRoute error: {:?}", error);
    }

    Ok(None)
}
