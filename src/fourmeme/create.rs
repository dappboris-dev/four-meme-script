use ethers::prelude::*;
use ethers::abi::Abi;
use ethers::types::Bytes;
use reqwest::{Client, multipart};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::{fs, sync::Arc, path::Path};
use anyhow::{Result, Context};
use crate::fourmeme::types::{CreateTokenRequest, NonceResponse, NonceRequest, VerifyInfo, LoginRequest, LoginResponse};

/// Prepare a createToken transaction (do NOT send yet)
pub async fn get_create_new_token_tx(
    wallet: LocalWallet,
    provider: Arc<Provider<Http>>,
    image_path: &str,
    config_path: &str,
) -> Result<TypedTransaction> {
    let client = Client::new();
    let address = wallet.address();
    let address_str = format!("{:?}", address);

    // --- Step 1: Get nonce ---
    let nonce_req = NonceRequest {
        account_address: &address_str,
        verify_type: "LOGIN",
        network_code: "BSC",
    };
    let nonce_res: NonceResponse = client
        .post("https://four.meme/meme-api/v1/private/user/nonce/generate")
        .json(&nonce_req)
        .send()
        .await?
        .json()
        .await?;
    if nonce_res.code != 0 {
        anyhow::bail!("Failed to get nonce: {:?}", nonce_res);
    }

    let message = format!("You are sign in Meme {}", nonce_res.data);
    let signature = wallet.sign_message(message).await?;

    // --- Step 2: Login ---
    let verify_info = VerifyInfo {
        address: &address_str,
        network_code: "BSC",
        signature: signature.to_string(),
        verify_type: "LOGIN",
    };
    let login_req = LoginRequest {
        region: "WEB",
        lang_type: "EN",
        login_ip: "",
        invite_code: "",
        verify_info,
        wallet_name: "MetaMask",
    };
    let login_res: LoginResponse = client
        .post("https://four.meme/meme-api/v1/private/user/login/dex")
        .json(&login_req)
        .send()
        .await?
        .json()
        .await?;
    let token = login_res.data.context("No access token returned from login")?;

    // --- Step 3: Upload image ---
    let path = Path::new(image_path);
    let file_bytes = tokio::fs::read(path).await?;
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("image.png");
    let form = multipart::Form::new()
        .part(
            "file",
            multipart::Part::bytes(file_bytes)
                .file_name(file_name.to_string())
                .mime_str("image/png")?,
        );
    let upload_res: Value = client
        .post("https://four.meme/meme-api/v1/private/token/upload")
        .header("meme-web-access", token.clone())
        .multipart(form)
        .send()
        .await?
        .json()
        .await?;
    let uploaded_url = upload_res["data"]
        .as_str()
        .context("No URL returned from upload")?;

    // --- Step 4: Read config metadata ---
    let mut payload: CreateTokenRequest = serde_json::from_str(&fs::read_to_string(config_path)?)?;
    payload.img_url = uploaded_url.to_string();

    // --- Step 5: Call create API to get createArg & signature ---
    let create_res: Value = client
        .post("https://four.meme/meme-api/v1/private/token/create")
        .header("meme-web-access", token)
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;
    let create_arg_hex = create_res["data"]["createArg"]
        .as_str()
        .context("No createArg returned")?
        .to_string();
    let sign_hex = create_res["data"]["signature"]
        .as_str()
        .context("No signature returned")?
        .to_string();

    // --- Step 6: Build ethers transaction (NOT sent) ---
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    let token_manager_addr: Address = "0x5c952063c7fc8610FFDB798152D69F0B9550762b".parse()?;
    let abi_json = fs::read_to_string("abi/TokenManager2.lite.abi")?;
    let abi: Abi = serde_json::from_str(&abi_json)?;
    let contract = Contract::new(token_manager_addr, abi, client.clone());

    let create_arg_bytes = Bytes::from(hex::decode(create_arg_hex.trim_start_matches("0x"))?);
    let sign_bytes = Bytes::from(hex::decode(sign_hex.trim_start_matches("0x"))?);

    let pre_sale_wei = ethers::utils::parse_ether(payload.pre_sale.as_str())?;
    let deploy_fee_wei = ethers::utils::parse_ether("0.01")?;
    let total_value_wei = pre_sale_wei + deploy_fee_wei;

    // Build transaction without sending
    let tx_call = contract
        .method::<_, H256>("createToken", (create_arg_bytes, sign_bytes))?
        .value(total_value_wei)
        .tx;

    Ok(tx_call)
}
