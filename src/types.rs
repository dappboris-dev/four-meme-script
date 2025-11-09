use ethers::types::Address;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletInfo {
    pub address: Address,
    pub private_key: String,
}
