//! Program-related RPC methods

use crate::{
    error::{Result, SolanaError},
    rpc::types::{RpcKeyedAccount, RpcProgramAccountsConfig},
    types::Pubkey,
};

/// Program-related RPC method implementations
pub struct ProgramMethods;

impl ProgramMethods {
    /// Get program accounts
    pub async fn get_program_accounts(
        client: &reqwest::Client,
        url: &str,
        program_id: &Pubkey,
        config: Option<RpcProgramAccountsConfig>,
    ) -> Result<Vec<RpcKeyedAccount>> {
        let method = "getProgramAccounts";
        let program_id_str = program_id.to_base58();
        
        let params = match config {
            Some(cfg) => {
                let mut map = serde_json::Map::new();
                
                // Add account config
                map.insert("encoding".to_string(), 
                    serde_json::json!(cfg.account_config.encoding.unwrap_or("base64".to_string())));
                
                if let Some(commitment) = &cfg.account_config.commitment {
                    map.insert("commitment".to_string(), serde_json::json!(commitment));
                }
                
                if let Some(data_slice) = &cfg.account_config.data_slice {
                    map.insert("dataSlice".to_string(), serde_json::json!({
                        "offset": data_slice.offset,
                        "length": data_slice.length,
                    }));
                }
                
                // Add filters if present
                if let Some(filters) = cfg.filters {
                    let filters_json = serde_json::to_value(filters)
                        .map_err(|e| SolanaError::Serialization(format!("Failed to serialize filters: {}", e)))?;
                    map.insert("filters".to_string(), filters_json);
                }
                
                // Add with_context if present
                if let Some(with_context) = cfg.with_context {
                    map.insert("withContext".to_string(), serde_json::json!(with_context));
                }
                
                serde_json::json!([program_id_str, map])
            },
            None => serde_json::json!([program_id_str])
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
        
        let accounts: Vec<RpcKeyedAccount> = serde_json::from_value(result.clone())
            .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize program accounts: {}", e)))?;
        
        Ok(accounts)
    }

    /// Get account info for a program
    pub async fn get_account_info(
        client: &reqwest::Client,
        url: &str,
        program_id: &Pubkey,
    ) -> Result<Option<crate::rpc::types::RpcAccount>> {
        // Reuse the account method implementation
        crate::rpc::methods::AccountMethods::get_account_info(client, url, program_id, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_program_accounts() {
        let client = reqwest::Client::new();
        let url = "https://api.mainnet-beta.solana.com";
        // Token program ID
        let program_id = Pubkey::from_base58("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

        // Create a config with a limit
        let mut config = RpcProgramAccountsConfig::default();
        config.account_config.encoding = Some("base64".to_string());
        
        // Add a filter for data size to limit results
        use crate::rpc::types::RpcFilterType;
        config.filters = Some(vec![RpcFilterType::DataSize(165)]);

        let result = ProgramMethods::get_program_accounts(&client, url, &program_id, Some(config)).await;
        assert!(result.is_ok());
        
        // Should have some accounts
        let accounts = result.unwrap();
        assert!(!accounts.is_empty());
    }
}