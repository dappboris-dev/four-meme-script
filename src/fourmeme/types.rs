use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct VerifyInfo<'a> {
    pub address: &'a str,
    #[serde(rename = "networkCode")]
    pub network_code: &'a str,
    pub signature: String,
    #[serde(rename = "verifyType")]
    pub verify_type: &'a str,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest<'a> {
    pub region: &'a str,
    #[serde(rename = "langType")]
    pub lang_type: &'a str,
    pub login_ip: &'a str,
    pub invite_code: &'a str,
    pub verify_info: VerifyInfo<'a>,
    pub wallet_name: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub code: i32,
    pub data: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NonceRequest<'a> {
    #[serde(rename = "accountAddress")]
    pub account_address: &'a str,
    pub verify_type: &'a str,
    pub network_code: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct NonceResponse {
    pub code: i32,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    pub desc: String,
    #[serde(rename = "imgUrl")]
    pub img_url: String,
    pub launch_time: u64,
    pub label: String,
    pub lp_trading_fee: f64,
    pub web_url: String,
    pub twitter_url: String,
    pub telegram_url: String,
    pub pre_sale: String,
    pub only_mpc: bool,
    pub raised_amount: u64,
    pub symbol: String,
}
