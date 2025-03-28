use super::{
    request::{RequestId, RpcRequest},
    response::*,
};
use crate::{Pubkey, Result, SolanaError};

/// A client for interacting with a Solana RPC node
#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
        }
    }

    /// Get the latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<([u8; 32], u64)> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "getLatestBlockhash".to_string(),
            params: vec![serde_json::json!({
                "commitment": "confirmed"
            })],
        };

        let response: RpcResponse<BlockhashResponse> = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::RpcError(e.to_string()))?
            .json()
            .await
            .map_err(|e| SolanaError::RpcError(e.to_string()))?;

        let blockhash = bs58::decode(&response.result.value.blockhash)
            .into_vec()
            .map_err(|_| SolanaError::RpcError("Invalid blockhash".to_string()))?;

        if blockhash.len() != 32 {
            return Err(SolanaError::RpcError(
                "Invalid blockhash length".to_string(),
            ));
        }

        Ok((
            blockhash.try_into().unwrap(),
            response.result.value.last_valid_block_height,
        ))
    }

    /// Submit a transaction
    pub async fn submit_transaction(&self, transaction: &[u8]) -> Result<String> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "sendTransaction".to_string(),
            params: vec![
                bs58::encode(transaction).into_string().into(),
                serde_json::json!({
                    "encoding": "base58",
                    "skip_preflight": true
                }),
            ],
        };

        let response: RpcResponse<String> = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::RpcError(e.to_string()))?
            .json()
            .await
            .map_err(|e| SolanaError::RpcError(e.to_string()))?;

        Ok(response.result)
    }

    /// Get account info
    pub async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<Vec<u8>>> {
        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(1),
            method: "getAccountInfo".to_string(),
            params: vec![
                pubkey.to_base58().into(),
                serde_json::json!({
                    "encoding": "base64"
                }),
            ],
        };

        let response: RpcResponse<AccountResponse> = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::RpcError(e.to_string()))?
            .json()
            .await
            .map_err(|e| SolanaError::RpcError(e.to_string()))?;

        if let Some(account_info) = response.result.value {
            if account_info.data.len() != 1 {
                return Err(SolanaError::RpcError("Invalid account data".to_string()));
            }

            let data = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &account_info.data[0],
            )
            .map_err(|_| SolanaError::RpcError("Invalid base64 data".to_string()))?;

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}
