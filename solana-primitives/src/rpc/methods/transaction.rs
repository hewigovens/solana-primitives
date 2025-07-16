//! Transaction-related RPC methods

use crate::{
    error::{Result, SolanaError},
    rpc::types::RpcSimulateTransactionResult,
    types::VersionedTransaction,
};

/// Transaction-related RPC method implementations
pub struct TransactionMethods;

impl TransactionMethods {
    /// Send a transaction
    pub async fn send_transaction(
        _client: &reqwest::Client,
        _url: &str,
        _transaction: &VersionedTransaction,
    ) -> Result<String> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "send_transaction not yet implemented".to_string()
        )))
    }

    /// Simulate a transaction
    pub async fn simulate_transaction(
        _client: &reqwest::Client,
        _url: &str,
        _transaction: &VersionedTransaction,
    ) -> Result<RpcSimulateTransactionResult> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "simulate_transaction not yet implemented".to_string()
        )))
    }

    /// Get transaction status
    pub async fn get_signature_status(
        _client: &reqwest::Client,
        _url: &str,
        _signature: &str,
    ) -> Result<Option<serde_json::Value>> {
        // TODO: Implement actual RPC call
        Err(SolanaError::Rpc(crate::error::RpcError::MethodNotFound(
            "get_signature_status not yet implemented".to_string()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{LegacyMessage, MessageHeader, CompiledInstruction, Pubkey, SignatureBytes};

    fn create_test_transaction() -> VersionedTransaction {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        };
        let account_keys = vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])];
        let recent_blockhash = [0u8; 32];
        let instructions = vec![CompiledInstruction {
            program_id_index: 1,
            accounts: vec![0],
            data: vec![],
        }];

        let message = LegacyMessage {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        };

        VersionedTransaction::Legacy {
            signatures: vec![SignatureBytes::new([0; 64])],
            message,
        }
    }

    #[tokio::test]
    async fn test_transaction_methods_placeholder() {
        let client = reqwest::Client::new();
        let url = "https://api.mainnet-beta.solana.com";
        let transaction = create_test_transaction();

        // These should all return not implemented errors for now
        let result = TransactionMethods::send_transaction(&client, url, &transaction).await;
        assert!(result.is_err());

        let result = TransactionMethods::simulate_transaction(&client, url, &transaction).await;
        assert!(result.is_err());

        let result = TransactionMethods::get_signature_status(&client, url, "test_signature").await;
        assert!(result.is_err());
    }
}