//! Mock RPC client for testing

#[cfg(feature = "testing")]
use crate::{
    error::{Result, SolanaError},
    rpc::types::{RpcAccount, RpcKeyedAccount, RpcSimulateTransactionResult},
    types::{Pubkey, VersionedTransaction},
};
#[cfg(feature = "testing")]
use std::collections::HashMap;

/// Mock RPC client for testing
#[cfg(feature = "testing")]
#[derive(Debug, Default)]
pub struct MockRpcClient {
    responses: HashMap<String, serde_json::Value>,
    expected_calls: Vec<(String, serde_json::Value)>,
    call_count: HashMap<String, usize>,
}

#[cfg(feature = "testing")]
impl MockRpcClient {
    /// Create a new mock RPC client
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a response for a specific method
    pub fn add_response(&mut self, method: &str, response: serde_json::Value) {
        self.responses.insert(method.to_string(), response);
    }

    /// Expect a specific call with parameters
    pub fn expect_call(&mut self, method: &str, params: serde_json::Value) {
        self.expected_calls.push((method.to_string(), params));
    }

    /// Get account information (mock)
    pub async fn get_account_info(&mut self, _pubkey: &Pubkey) -> Result<Option<RpcAccount>> {
        let method = "getAccountInfo";
        self.increment_call_count(method);

        if let Some(response) = self.responses.get(method) {
            if response.is_null() {
                Ok(None)
            } else {
                let account: RpcAccount = serde_json::from_value(response.clone())
                    .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize mock response: {}", e)))?;
                Ok(Some(account))
            }
        } else {
            // Default mock response
            Ok(Some(RpcAccount {
                lamports: 1000000000, // 1 SOL
                data: vec!["".to_string(), "base64".to_string()],
                owner: "11111111111111111111111111111111".to_string(),
                executable: false,
                rent_epoch: 361,
            }))
        }
    }

    /// Get balance (mock)
    pub async fn get_balance(&mut self, _pubkey: &Pubkey) -> Result<u64> {
        let method = "getBalance";
        self.increment_call_count(method);

        if let Some(response) = self.responses.get(method) {
            let balance: u64 = serde_json::from_value(response.clone())
                .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize mock response: {}", e)))?;
            Ok(balance)
        } else {
            // Default mock response
            Ok(1000000000) // 1 SOL
        }
    }

    /// Get program accounts (mock)
    pub async fn get_program_accounts(&mut self, _program_id: &Pubkey) -> Result<Vec<RpcKeyedAccount>> {
        let method = "getProgramAccounts";
        self.increment_call_count(method);

        if let Some(response) = self.responses.get(method) {
            let accounts: Vec<RpcKeyedAccount> = serde_json::from_value(response.clone())
                .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize mock response: {}", e)))?;
            Ok(accounts)
        } else {
            // Default mock response
            Ok(vec![])
        }
    }

    /// Send transaction (mock)
    pub async fn send_transaction(&mut self, _transaction: &VersionedTransaction) -> Result<String> {
        let method = "sendTransaction";
        self.increment_call_count(method);

        if let Some(response) = self.responses.get(method) {
            let signature: String = serde_json::from_value(response.clone())
                .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize mock response: {}", e)))?;
            Ok(signature)
        } else {
            // Default mock response
            Ok("5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW".to_string())
        }
    }

    /// Simulate transaction (mock)
    pub async fn simulate_transaction(&mut self, _transaction: &VersionedTransaction) -> Result<RpcSimulateTransactionResult> {
        let method = "simulateTransaction";
        self.increment_call_count(method);

        if let Some(response) = self.responses.get(method) {
            let result: RpcSimulateTransactionResult = serde_json::from_value(response.clone())
                .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize mock response: {}", e)))?;
            Ok(result)
        } else {
            // Default mock response
            Ok(RpcSimulateTransactionResult {
                err: None,
                logs: Some(vec!["Program log: Hello, world!".to_string()]),
                accounts: None,
                units_consumed: Some(1000),
                return_data: None,
            })
        }
    }

    /// Get the number of times a method was called
    pub fn call_count(&self, method: &str) -> usize {
        self.call_count.get(method).copied().unwrap_or(0)
    }

    /// Reset all call counts
    pub fn reset_call_counts(&mut self) {
        self.call_count.clear();
    }

    /// Verify that all expected calls were made
    pub fn verify_expected_calls(&self) -> Result<()> {
        for (method, _params) in &self.expected_calls {
            if self.call_count(method) == 0 {
                return Err(SolanaError::Transaction(
                    format!("Expected call to {} was not made", method)
                ));
            }
        }
        Ok(())
    }

    fn increment_call_count(&mut self, method: &str) {
        *self.call_count.entry(method.to_string()).or_insert(0) += 1;
    }
}

#[cfg(test)]
#[cfg(feature = "testing")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client_get_balance() {
        let mut client = MockRpcClient::new();
        let pubkey = Pubkey::new([1; 32]);

        // Test default response
        let balance = client.get_balance(&pubkey).await.unwrap();
        assert_eq!(balance, 1000000000);
        assert_eq!(client.call_count("getBalance"), 1);

        // Test custom response
        client.add_response("getBalance", serde_json::json!(2000000000u64));
        let balance = client.get_balance(&pubkey).await.unwrap();
        assert_eq!(balance, 2000000000);
        assert_eq!(client.call_count("getBalance"), 2);
    }

    #[tokio::test]
    async fn test_mock_client_get_account_info() {
        let mut client = MockRpcClient::new();
        let pubkey = Pubkey::new([1; 32]);

        // Test default response
        let account = client.get_account_info(&pubkey).await.unwrap();
        assert!(account.is_some());
        assert_eq!(client.call_count("getAccountInfo"), 1);

        // Test null response
        client.add_response("getAccountInfo", serde_json::Value::Null);
        let account = client.get_account_info(&pubkey).await.unwrap();
        assert!(account.is_none());
        assert_eq!(client.call_count("getAccountInfo"), 2);
    }

    #[tokio::test]
    async fn test_mock_client_send_transaction() {
        let mut client = MockRpcClient::new();
        
        // Create a dummy transaction
        use crate::types::{LegacyMessage, MessageHeader, CompiledInstruction, SignatureBytes};
        
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

        let transaction = VersionedTransaction::Legacy {
            signatures: vec![SignatureBytes::new([0; 64])],
            message,
        };

        // Test default response
        let signature = client.send_transaction(&transaction).await.unwrap();
        assert!(!signature.is_empty());
        assert_eq!(client.call_count("sendTransaction"), 1);
    }

    #[tokio::test]
    async fn test_mock_client_expected_calls() {
        let mut client = MockRpcClient::new();
        let pubkey = Pubkey::new([1; 32]);

        // Set up expected call
        client.expect_call("getBalance", serde_json::json!([pubkey.to_base58()]));

        // Make the call
        let _balance = client.get_balance(&pubkey).await.unwrap();

        // Verify expected calls
        let result = client.verify_expected_calls();
        assert!(result.is_ok());
    }
}