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
        _client: &reqwest::Client,
        _url: &str,
        _pubkey: &Pubkey,
        _config: Option<RpcAccountInfoConfig>,
    ) -> Result<Option<RpcAccount>> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "get_account_info not yet implemented".to_string()
        )))
    }

    /// Get multiple accounts information
    pub async fn get_multiple_accounts(
        _client: &reqwest::Client,
        _url: &str,
        _pubkeys: &[Pubkey],
        _config: Option<RpcAccountInfoConfig>,
    ) -> Result<Vec<Option<RpcAccount>>> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "get_multiple_accounts not yet implemented".to_string()
        )))
    }

    /// Get account balance
    pub async fn get_balance(
        _client: &reqwest::Client,
        _url: &str,
        _pubkey: &Pubkey,
    ) -> Result<u64> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "get_balance not yet implemented".to_string()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_methods_placeholder() {
        let client = reqwest::Client::new();
        let url = "https://api.mainnet-beta.solana.com";
        let pubkey = Pubkey::new([1; 32]);

        // These should all return not implemented errors for now
        let result = AccountMethods::get_account_info(&client, url, &pubkey, None).await;
        assert!(result.is_err());

        let result = AccountMethods::get_multiple_accounts(&client, url, &[pubkey], None).await;
        assert!(result.is_err());

        let result = AccountMethods::get_balance(&client, url, &pubkey).await;
        assert!(result.is_err());
    }
}