//! Account-related RPC methods

use crate::{
    error::{Result, SolanaError},
    rpc::types::{RpcAccount, RpcAccountInfoConfig},
    types::Pubkey,
};

/// Account-related RPC method implementations
pub struct AccountMethods;

impl AccountMethods {
    /// Get account information
    pub async fn get_account_info(
        client: &reqwest::Client,
        url: &str,
        pubkey: &Pubkey,
        config: Option<RpcAccountInfoConfig>,
    ) -> Result<Option<RpcAccount>> {
        let method = "getAccountInfo";
        let pubkey_str = pubkey.to_base58();
        
        let params = match config {
            Some(cfg) => {
                let mut map = serde_json::Map::new();
                map.insert("encoding".to_string(), serde_json::json!(cfg.encoding.unwrap_or("base64".to_string())));
                if let Some(commitment) = cfg.commitment {
                    map.insert("commitment".to_string(), serde_json::json!(commitment));
                }
                if let Some(data_slice) = &cfg.data_slice {
                    map.insert("dataSlice".to_string(), serde_json::json!({
                        "offset": data_slice.offset,
                        "length": data_slice.length,
                    }));
                }
                serde_json::json!([pubkey_str, map])
            },
            None => serde_json::json!([pubkey_str])
        };
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        
        let response = client.post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::Network(e))?;
        
        let response_text = response.text().await
            .map_err(|e| SolanaError::Network(e))?;
        
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| SolanaError::Serialization(format!("Failed to parse JSON response: {}", e)))?;
        
        if let Some(error) = response_json.get("error") {
            return Err(SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                format!("RPC error: {}", error)
            )));
        }
        
        let result = response_json.get("result")
            .ok_or_else(|| SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                "Missing 'result' in response".to_string()
            )))?;
        
        if result.is_null() {
            return Ok(None);
        }
        
        let value = result.get("value");
        if value.is_none() || value.unwrap().is_null() {
            return Ok(None);
        }
        
        let account: RpcAccount = serde_json::from_value(value.unwrap().clone())
            .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize account: {}", e)))?;
        
        Ok(Some(account))
    }

    /// Get multiple accounts information
    pub async fn get_multiple_accounts(
        client: &reqwest::Client,
        url: &str,
        pubkeys: &[Pubkey],
        config: Option<RpcAccountInfoConfig>,
    ) -> Result<Vec<Option<RpcAccount>>> {
        let method = "getMultipleAccounts";
        let pubkey_strs: Vec<String> = pubkeys.iter().map(|p| p.to_base58()).collect();
        
        let params = match config {
            Some(cfg) => {
                let mut map = serde_json::Map::new();
                map.insert("encoding".to_string(), serde_json::json!(cfg.encoding.unwrap_or("base64".to_string())));
                if let Some(commitment) = cfg.commitment {
                    map.insert("commitment".to_string(), serde_json::json!(commitment));
                }
                if let Some(data_slice) = &cfg.data_slice {
                    map.insert("dataSlice".to_string(), serde_json::json!({
                        "offset": data_slice.offset,
                        "length": data_slice.length,
                    }));
                }
                serde_json::json!([pubkey_strs, map])
            },
            None => serde_json::json!([pubkey_strs])
        };
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        
        let response = client.post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::Network(e))?;
        
        let response_text = response.text().await
            .map_err(|e| SolanaError::Network(e))?;
        
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| SolanaError::Serialization(format!("Failed to parse JSON response: {}", e)))?;
        
        if let Some(error) = response_json.get("error") {
            return Err(SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                format!("RPC error: {}", error)
            )));
        }
        
        let result = response_json.get("result")
            .ok_or_else(|| SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                "Missing 'result' in response".to_string()
            )))?;
        
        let value = result.get("value")
            .ok_or_else(|| SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                "Missing 'value' in result".to_string()
            )))?;
        
        let accounts: Vec<Option<RpcAccount>> = serde_json::from_value(value.clone())
            .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize accounts: {}", e)))?;
        
        Ok(accounts)
    }

    /// Get account balance
    pub async fn get_balance(
        client: &reqwest::Client,
        url: &str,
        pubkey: &Pubkey,
    ) -> Result<u64> {
        let method = "getBalance";
        let pubkey_str = pubkey.to_base58();
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": [pubkey_str],
        });
        
        let response = client.post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| SolanaError::Network(e))?;
        
        let response_text = response.text().await
            .map_err(|e| SolanaError::Network(e))?;
        
        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| SolanaError::Serialization(format!("Failed to parse JSON response: {}", e)))?;
        
        if let Some(error) = response_json.get("error") {
            return Err(SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                format!("RPC error: {}", error)
            )));
        }
        
        let result = response_json.get("result")
            .ok_or_else(|| SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                "Missing 'result' in response".to_string()
            )))?;
        
        let value = result.get("value")
            .ok_or_else(|| SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                "Missing 'value' in result".to_string()
            )))?;
        
        let balance: u64 = serde_json::from_value(value.clone())
            .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize balance: {}", e)))?;
        
        Ok(balance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_account_info() {
        let client = reqwest::Client::new();
        let url = "https://api.mainnet-beta.solana.com";
        let pubkey = Pubkey::from_base58("SysvarC1ock11111111111111111111111111111111").unwrap();

        let result = AccountMethods::get_account_info(&client, url, &pubkey, None).await;
        assert!(result.is_ok());
        
        let account = result.unwrap();
        assert!(account.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_balance() {
        let client = reqwest::Client::new();
        let url = "https://api.mainnet-beta.solana.com";
        let pubkey = Pubkey::from_base58("SysvarC1ock11111111111111111111111111111111").unwrap();

        let result = AccountMethods::get_balance(&client, url, &pubkey).await;
        assert!(result.is_ok());
        
        let balance = result.unwrap();
        assert!(balance > 0);
    }
}