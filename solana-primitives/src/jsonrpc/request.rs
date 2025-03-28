use serde::{Deserialize, Serialize};

/// Represents a JSON-RPC request ID which can be either a number or a string
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(u64),
    String(String),
}

/// Represents a JSON-RPC request to a Solana node
#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    pub params: Vec<serde_json::Value>,
}
