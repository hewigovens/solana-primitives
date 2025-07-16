//! Enhanced RPC client

use crate::{
    error::Result,
    rpc::{
        methods::{AccountMethods, ProgramMethods, TransactionMethods},
        types::{
            RpcAccount, RpcAccountInfoConfig, RpcConfig, RpcKeyedAccount,
            RpcProgramAccountsConfig, RpcSimulateTransactionResult,
        },
    },
    types::{Pubkey, VersionedTransaction},
};

/// Enhanced RPC client for Solana
#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
    config: RpcConfig,
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
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

    /// Submit a transaction (backward compatibility alias for send_transaction)
    pub async fn submit_transaction(&self, transaction: &[u8]) -> Result<String> {
        // For now, this is a placeholder that calls the legacy RPC client
        // In a full implementation, this would convert the bytes to a VersionedTransaction
        // and call send_transaction, or implement the actual RPC call
        use crate::jsonrpc::RpcClient as LegacyRpcClient;
        let legacy_client = LegacyRpcClient::new(self.url.clone());
        legacy_client.submit_transaction(transaction).await
    }

    /// Get the latest blockhash (backward compatibility method)
    pub async fn get_latest_blockhash(&self) -> Result<([u8; 32], u64)> {
        // For now, this is a placeholder that calls the legacy RPC client
        // In a full implementation, this would be implemented using the new RPC method structure
        use crate::jsonrpc::RpcClient as LegacyRpcClient;
        let legacy_client = LegacyRpcClient::new(self.url.clone());
        legacy_client.get_latest_blockhash().await
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
    async fn test_rpc_methods_placeholder() {
        let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let pubkey = Pubkey::new([1; 32]);

        // These should all return not implemented errors for now
        let result = client.get_account_info(&pubkey).await;
        assert!(result.is_err());

        let result = client.get_balance(&pubkey).await;
        assert!(result.is_err());

        let result = client.get_program_accounts(&pubkey).await;
        assert!(result.is_err());
    }
}