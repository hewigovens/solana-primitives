use super::request::RequestId;
use serde::{Deserialize, Serialize};

/// Generic RPC response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: RequestId,
    pub result: T,
}

/// Response for a blockhash request
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockhashResponse {
    pub value: BlockhashValue,
}

/// Value part of a blockhash response
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockhashValue {
    pub blockhash: String,
    pub last_valid_block_height: u64,
}

/// Account information response
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub data: Vec<String>,
    pub executable: bool,
    pub lamports: u64,
    pub owner: String,
    pub rent_epoch: u64,
}

/// Response for an account info request
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub value: Option<AccountInfo>,
}
