//! RPC-specific types and configurations

use crate::types::Pubkey;
use serde::{Deserialize, Serialize};

/// RPC configuration options
#[derive(Debug, Clone, Default)]
pub struct RpcConfig {
    /// Request timeout in seconds
    pub timeout: Option<u64>,
    /// Maximum number of retries
    pub max_retries: Option<u32>,
    /// Retry delay in milliseconds
    pub retry_delay: Option<u64>,
    /// Custom headers
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// Account information configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcAccountInfoConfig {
    /// Encoding format for account data
    pub encoding: Option<String>,
    /// Commitment level
    pub commitment: Option<String>,
    /// Data slice configuration
    pub data_slice: Option<RpcDataSlice>,
    /// Minimum context slot
    pub min_context_slot: Option<u64>,
}

/// Data slice configuration for partial account data retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcDataSlice {
    /// Offset in bytes
    pub offset: usize,
    /// Length in bytes
    pub length: usize,
}

/// Program accounts configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcProgramAccountsConfig {
    /// Account filters
    pub filters: Option<Vec<RpcFilterType>>,
    /// Account configuration
    pub account_config: RpcAccountInfoConfig,
    /// Whether to include context
    pub with_context: Option<bool>,
}

/// Filter types for program account queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcFilterType {
    /// Data size filter
    DataSize(u64),
    /// Memcmp filter
    Memcmp {
        /// Offset in account data
        offset: usize,
        /// Bytes to match (base58 encoded)
        bytes: String,
        /// Encoding format
        encoding: Option<String>,
    },
}

/// Transaction simulation configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpcSimulateTransactionConfig {
    /// Whether to replace recent blockhash
    pub replace_recent_blockhash: Option<bool>,
    /// Commitment level
    pub commitment: Option<String>,
    /// Encoding format
    pub encoding: Option<String>,
    /// Accounts to return
    pub accounts: Option<RpcSimulateTransactionAccountsConfig>,
    /// Minimum context slot
    pub min_context_slot: Option<u64>,
}

/// Account configuration for transaction simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSimulateTransactionAccountsConfig {
    /// Encoding format for account data
    pub encoding: Option<String>,
    /// Addresses to return
    pub addresses: Vec<String>,
}

/// Keyed account from RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcKeyedAccount {
    /// Account public key
    pub pubkey: Pubkey,
    /// Account information
    pub account: RpcAccount,
}

/// Account information from RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAccount {
    /// Account balance in lamports
    pub lamports: u64,
    /// Account data
    pub data: Vec<String>,
    /// Account owner
    pub owner: String,
    /// Whether account is executable
    pub executable: bool,
    /// Rent epoch
    pub rent_epoch: u64,
}

/// Transaction simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSimulateTransactionResult {
    /// Transaction error if any
    pub err: Option<serde_json::Value>,
    /// Transaction logs
    pub logs: Option<Vec<String>>,
    /// Accounts after simulation
    pub accounts: Option<Vec<Option<RpcAccount>>>,
    /// Compute units consumed
    pub units_consumed: Option<u64>,
    /// Return data from transaction
    pub return_data: Option<RpcTransactionReturnData>,
}

/// Return data from transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcTransactionReturnData {
    /// Program ID that returned the data
    pub program_id: String,
    /// Returned data (base64 encoded)
    pub data: (String, String),
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_config_default() {
        let config = RpcConfig::default();
        assert!(config.timeout.is_none());
        assert!(config.max_retries.is_none());
        assert!(config.retry_delay.is_none());
        assert!(config.headers.is_none());
    }

    #[test]
    fn test_account_info_config_serialization() {
        let config = RpcAccountInfoConfig {
            encoding: Some("base64".to_string()),
            commitment: Some("confirmed".to_string()),
            data_slice: Some(RpcDataSlice {
                offset: 0,
                length: 32,
            }),
            min_context_slot: Some(100),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RpcAccountInfoConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.encoding, deserialized.encoding);
        assert_eq!(config.commitment, deserialized.commitment);
    }

    #[test]
    fn test_filter_type_serialization() {
        let data_size_filter = RpcFilterType::DataSize(165);
        let json = serde_json::to_string(&data_size_filter).unwrap();
        assert_eq!(json, "165");

        let memcmp_filter = RpcFilterType::Memcmp {
            offset: 0,
            bytes: "test".to_string(),
            encoding: Some("base58".to_string()),
        };
        let json = serde_json::to_string(&memcmp_filter).unwrap();
        assert!(json.contains("offset"));
        assert!(json.contains("bytes"));
    }
}