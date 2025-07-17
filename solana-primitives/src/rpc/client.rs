use crate::{
    error::Result,
    rpc::{methods::{AccountMethods, ProgramMethods, TransactionMethods}, types::*},
    types::{Pubkey, VersionedTransaction},
};
use reqwest::Client;

/// A client for interacting with Solana's RPC API
pub struct RpcClient {
    /// The URL of the RPC endpoint
    url: String,
    /// The HTTP client
    client: Client,
    /// RPC configuration
    config: RpcConfig,
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::new(),
            config: RpcConfig::default(),
        }
    }

    /// Create a new RPC client with configuration
    pub fn new_with_config(url: String, config: RpcConfig) -> Self {
        let mut client_builder = reqwest::Client::builder();

        if let Some(timeout) = config.timeout {
            client_builder = client_builder.timeout(std::time::Duration::from_secs(timeout));
        }

        let client = client_builder.build().unwrap_or_else(|_| reqwest::Client::new());

        Self { url, client, config }
    }

    /// Get the latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<([u8; 32], u64)> {
        let response = self
            .client
            .post(&self.url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getLatestBlockhash",
                "params": [{"commitment": "confirmed"}]
            }))
            .send()
            .await
            .map_err(crate::error::SolanaError::Network)?;

        let response_text = response.text().await
            .map_err(crate::error::SolanaError::Network)?;

        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| crate::error::SolanaError::Serialization(format!("Failed to parse JSON response: {e}")))?;

        if let Some(error) = response_json.get("error") {
            return Err(crate::error::SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                format!("RPC error: {error}")
            )));
        }

        let blockhash_str = response_json["result"]["value"]["blockhash"]
            .as_str()
            .ok_or_else(|| {
                crate::error::SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                    "Invalid blockhash response".to_string()
                ))
            })?;

        let last_valid_block_height = response_json["result"]["value"]["lastValidBlockHeight"]
            .as_u64()
            .ok_or_else(|| {
                crate::error::SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                    "Invalid lastValidBlockHeight response".to_string()
                ))
            })?;

        let blockhash_bytes = bs58::decode(blockhash_str)
            .into_vec()
            .map_err(|e| crate::error::SolanaError::Serialization(e.to_string()))?;

        if blockhash_bytes.len() != 32 {
            return Err(crate::error::SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                "Invalid blockhash length".to_string()
            )));
        }

        let mut blockhash = [0u8; 32];
        blockhash.copy_from_slice(&blockhash_bytes);

        Ok((blockhash, last_valid_block_height))
    }

    /// Submit a transaction (legacy method for backward compatibility)
    pub async fn submit_transaction(&self, transaction_bytes: &[u8]) -> Result<String> {
        let base64_tx = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, transaction_bytes);

        let response = self
            .client
            .post(&self.url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "sendTransaction",
                "params": [base64_tx],
            }))
            .send()
            .await
            .map_err(crate::error::SolanaError::Network)?;

        let response_text = response.text().await
            .map_err(crate::error::SolanaError::Network)?;

        let response_json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| crate::error::SolanaError::Serialization(format!("Failed to parse JSON response: {e}")))?;

        if let Some(error) = response_json.get("error") {
            return Err(crate::error::SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                format!("RPC error: {error}")
            )));
        }

        let signature = response_json["result"]
            .as_str()
            .ok_or_else(|| {
                crate::error::SolanaError::Rpc(crate::error::RpcError::InvalidRequest(
                    "Invalid signature response".to_string()
                ))
            })?
            .to_string();

        Ok(signature)
    }

    /// Get account information
    pub async fn get_account_info(
        &self,
        pubkey: &Pubkey,
    ) -> Result<Option<RpcAccount>> {
        self.get_account_info_with_config(pubkey, None).await
    }

    /// Get account information with configuration
    pub async fn get_account_info_with_config(
        &self,
        pubkey: &Pubkey,
        config: Option<RpcAccountInfoConfig>,
    ) -> Result<Option<RpcAccount>> {
        AccountMethods::get_account_info(&self.client, &self.url, pubkey, config).await
    }

    /// Get multiple accounts information
    pub async fn get_multiple_accounts(
        &self,
        pubkeys: &[Pubkey],
    ) -> Result<Vec<Option<RpcAccount>>> {
        self.get_multiple_accounts_with_config(pubkeys, None).await
    }

    /// Get multiple accounts information with configuration
    pub async fn get_multiple_accounts_with_config(
        &self,
        pubkeys: &[Pubkey],
        config: Option<RpcAccountInfoConfig>,
    ) -> Result<Vec<Option<RpcAccount>>> {
        AccountMethods::get_multiple_accounts(&self.client, &self.url, pubkeys, config).await
    }

    /// Get account balance
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        AccountMethods::get_balance(&self.client, &self.url, pubkey).await
    }

    /// Get program accounts
    pub async fn get_program_accounts(
        &self,
        program_id: &Pubkey,
    ) -> Result<Vec<RpcKeyedAccount>> {
        self.get_program_accounts_with_config(program_id, None).await
    }

    /// Get program accounts with configuration
    pub async fn get_program_accounts_with_config(
        &self,
        program_id: &Pubkey,
        config: Option<RpcProgramAccountsConfig>,
    ) -> Result<Vec<RpcKeyedAccount>> {
        ProgramMethods::get_program_accounts(&self.client, &self.url, program_id, config).await
    }

    /// Send a transaction
    pub async fn send_transaction(&self, transaction: &VersionedTransaction) -> Result<String> {
        TransactionMethods::send_transaction(&self.client, &self.url, transaction).await
    }

    /// Simulate a transaction
    pub async fn simulate_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<RpcSimulateTransactionResult> {
        TransactionMethods::simulate_transaction(&self.client, &self.url, transaction).await
    }

    /// Get signature status
    pub async fn get_signature_status(
        &self,
        signature: &str,
    ) -> Result<Option<serde_json::Value>> {
        TransactionMethods::get_signature_status(&self.client, &self.url, signature).await
    }

    /// Get the RPC URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the client configuration
    pub fn config(&self) -> &RpcConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_client_creation() {
        let url = "https://api.mainnet-beta.solana.com".to_string();
        let client = RpcClient::new(url.clone());

        assert_eq!(client.url(), &url);
    }

    #[test]
    fn test_rpc_client_with_config() {
        let url = "https://api.mainnet-beta.solana.com".to_string();
        let config = RpcConfig {
            timeout: Some(30),
            max_retries: Some(3),
            retry_delay: Some(1000),
            headers: None,
        };

        let client = RpcClient::new_with_config(url.clone(), config.clone());

        assert_eq!(client.url(), &url);
        assert_eq!(client.config().timeout, config.timeout);
        assert_eq!(client.config().max_retries, config.max_retries);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_latest_blockhash() {
        let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let result = client.get_latest_blockhash().await;
        assert!(result.is_ok());
    }
}
