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
        client: &reqwest::Client,
        url: &str,
        transaction: &VersionedTransaction,
    ) -> Result<String> {
        // Serialize the transaction
        let tx_bytes = match transaction {
            VersionedTransaction::Legacy { .. } => {
                transaction.serialize_legacy()
                    .map_err(|e| SolanaError::Serialization(format!("Failed to serialize transaction: {}", e)))?                
            },
            VersionedTransaction::V0 { .. } => {
                transaction.serialize_versioned()
                    .map_err(|e| SolanaError::Serialization(format!("Failed to serialize transaction: {}", e)))?                
            },
        };
        
        // Encode as base64
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes);
        
        let method = "sendTransaction";
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": [tx_base64],
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
        
        let signature: String = serde_json::from_value(result.clone())
            .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize signature: {}", e)))?;
        
        Ok(signature)
    }

    /// Simulate a transaction
    pub async fn simulate_transaction(
        client: &reqwest::Client,
        url: &str,
        transaction: &VersionedTransaction,
    ) -> Result<RpcSimulateTransactionResult> {
        // Serialize the transaction
        let tx_bytes = match transaction {
            VersionedTransaction::Legacy { .. } => {
                transaction.serialize_legacy()
                    .map_err(|e| SolanaError::Serialization(format!("Failed to serialize transaction: {}", e)))?                
            },
            VersionedTransaction::V0 { .. } => {
                transaction.serialize_versioned()
                    .map_err(|e| SolanaError::Serialization(format!("Failed to serialize transaction: {}", e)))?                
            },
        };
        
        // Encode as base64
        let tx_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes);
        
        let method = "simulateTransaction";
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": [tx_base64, {"encoding": "base64"}],
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
        
        let simulation_result: RpcSimulateTransactionResult = serde_json::from_value(value.clone())
            .map_err(|e| SolanaError::Serialization(format!("Failed to deserialize simulation result: {}", e)))?;
        
        Ok(simulation_result)
    }

    /// Get transaction status
    pub async fn get_signature_status(
        client: &reqwest::Client,
        url: &str,
        signature: &str,
    ) -> Result<Option<serde_json::Value>> {
        let method = "getSignatureStatuses";
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": [[signature]],
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
        
        if value.is_array() && value.as_array().unwrap().is_empty() {
            return Ok(None);
        }
        
        if value.is_array() && value.as_array().unwrap()[0].is_null() {
            return Ok(None);
        }
        
        Ok(Some(value[0].clone()))
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
    #[ignore] // This test would require a valid transaction and blockhash
    async fn test_send_transaction() {
        let client = reqwest::Client::new();
        let url = "https://api.devnet.solana.com"; // Use devnet for testing
        let transaction = create_test_transaction();

        let result = TransactionMethods::send_transaction(&client, url, &transaction).await;
        // This will likely fail with an invalid blockhash error, but we're just testing the method structure
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // This test would require a valid transaction
    async fn test_simulate_transaction() {
        let client = reqwest::Client::new();
        let url = "https://api.devnet.solana.com"; // Use devnet for testing
        let transaction = create_test_transaction();

        let result = TransactionMethods::simulate_transaction(&client, url, &transaction).await;
        // This will likely fail with an invalid blockhash error, but we're just testing the method structure
        assert!(result.is_err());
    }
}