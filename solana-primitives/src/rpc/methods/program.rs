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
        _client: &reqwest::Client,
        _url: &str,
        _program_id: &Pubkey,
        _config: Option<RpcProgramAccountsConfig>,
    ) -> Result<Vec<RpcKeyedAccount>> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "get_program_accounts not yet implemented".to_string()
        )))
    }

    /// Get account info for a program
    pub async fn get_account_info(
        _client: &reqwest::Client,
        _url: &str,
        _program_id: &Pubkey,
    ) -> Result<Option<crate::rpc::types::RpcAccount>> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "get_account_info not yet implemented".to_string()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_program_methods_placeholder() {
        let client = reqwest::Client::new();
        let url = "https://api.mainnet-beta.solana.com";
        let program_id = Pubkey::new([1; 32]);

        // These should all return not implemented errors for now
        let result = ProgramMethods::get_program_accounts(&client, url, &program_id, None).await;
        assert!(result.is_err());

        let result = ProgramMethods::get_account_info(&client, url, &program_id).await;
        assert!(result.is_err());
    }
}