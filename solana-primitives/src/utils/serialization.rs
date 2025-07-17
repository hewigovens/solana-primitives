//! Enhanced serialization helpers

use crate::error::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Serialization utilities
pub struct SerializationUtils;

impl SerializationUtils {
    /// Serialize to Borsh format
    pub fn to_borsh<T: BorshSerialize>(data: &T) -> Result<Vec<u8>> {
        borsh::to_vec(data)
            .map_err(|e| SolanaError::Serialization(format!("Borsh serialization failed: {e}")))
    }

    /// Deserialize from Borsh format
    pub fn from_borsh<T: BorshDeserialize>(data: &[u8]) -> Result<T> {
        T::try_from_slice(data)
            .map_err(|e| SolanaError::Serialization(format!("Borsh deserialization failed: {e}")))
    }

    /// Serialize to JSON format
    pub fn to_json<T: Serialize>(data: &T) -> Result<String> {
        serde_json::to_string(data)
            .map_err(|e| SolanaError::Serialization(format!("JSON serialization failed: {e}")))
    }

    /// Serialize to pretty JSON format
    pub fn to_json_pretty<T: Serialize>(data: &T) -> Result<String> {
        serde_json::to_string_pretty(data)
            .map_err(|e| SolanaError::Serialization(format!("JSON serialization failed: {e}")))
    }

    /// Deserialize from JSON format
    pub fn from_json<T: for<'de> Deserialize<'de>>(data: &str) -> Result<T> {
        serde_json::from_str(data)
            .map_err(|e| SolanaError::Serialization(format!("JSON deserialization failed: {e}")))
    }

    /// Convert bytes to base64 string
    pub fn to_base64(data: &[u8]) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
    }

    /// Convert base64 string to bytes
    pub fn from_base64(data: &str) -> Result<Vec<u8>> {
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)
            .map_err(|e| SolanaError::Serialization(format!("Base64 decoding failed: {e}")))
    }

    /// Convert bytes to base58 string
    pub fn to_base58(data: &[u8]) -> String {
        bs58::encode(data).into_string()
    }

    /// Convert base58 string to bytes
    pub fn from_base58(data: &str) -> Result<Vec<u8>> {
        bs58::decode(data)
            .into_vec()
            .map_err(|e| SolanaError::Serialization(format!("Base58 decoding failed: {e}")))
    }

    /// Convert bytes to hex string
    pub fn to_hex(data: &[u8]) -> String {
        hex::encode(data)
    }

    /// Convert hex string to bytes
    pub fn from_hex(data: &str) -> Result<Vec<u8>> {
        hex::decode(data)
            .map_err(|e| SolanaError::Serialization(format!("Hex decoding failed: {e}")))
    }
}

/// Compact array serialization helpers (for Solana's compact-u16 encoding)
pub struct CompactArrayUtils;

impl CompactArrayUtils {
    /// Encode length as compact-u16 bytes
    pub fn encode_length(len: usize) -> Result<Vec<u8>> {
        crate::encode_length_to_compact_u16_bytes(len)
            .map_err(|e| SolanaError::Serialization(e.to_string()))
    }

    /// Decode compact-u16 length from bytes
    pub fn decode_length(data: &[u8]) -> Result<(usize, usize)> {
        crate::decode_compact_u16_len(data)
            .map_err(|e| SolanaError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
    struct TestStruct {
        value: u32,
        name: String,
    }

    #[test]
    fn test_borsh_serialization() {
        let data = TestStruct {
            value: 42,
            name: "test".to_string(),
        };

        let serialized = SerializationUtils::to_borsh(&data).unwrap();
        let deserialized: TestStruct = SerializationUtils::from_borsh(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_json_serialization() {
        let data = TestStruct {
            value: 42,
            name: "test".to_string(),
        };

        let json = SerializationUtils::to_json(&data).unwrap();
        let deserialized: TestStruct = SerializationUtils::from_json(&json).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"hello world";
        let encoded = SerializationUtils::to_base64(data);
        let decoded = SerializationUtils::from_base64(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_base58_encoding() {
        let data = b"hello world";
        let encoded = SerializationUtils::to_base58(data);
        let decoded = SerializationUtils::from_base58(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"hello world";
        let encoded = SerializationUtils::to_hex(data);
        let decoded = SerializationUtils::from_hex(&encoded).unwrap();

        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_compact_length_encoding() {
        // Test single byte encoding
        let encoded = CompactArrayUtils::encode_length(127).unwrap();
        assert_eq!(encoded, vec![127]);
        let (decoded, consumed) = CompactArrayUtils::decode_length(&encoded).unwrap();
        assert_eq!(decoded, 127);
        assert_eq!(consumed, 1);

        // Test two byte encoding
        let encoded = CompactArrayUtils::encode_length(128).unwrap();
        assert_eq!(encoded.len(), 2);
        let (decoded, consumed) = CompactArrayUtils::decode_length(&encoded).unwrap();
        assert_eq!(decoded, 128);
        assert_eq!(consumed, 2);

        // Test three byte encoding
        let encoded = CompactArrayUtils::encode_length(16384).unwrap();
        assert_eq!(encoded.len(), 3);
        let (decoded, consumed) = CompactArrayUtils::decode_length(&encoded).unwrap();
        assert_eq!(decoded, 16384);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_compact_length_edge_cases() {
        // Test empty data
        let result = CompactArrayUtils::decode_length(&[]);
        assert!(result.is_err());

        // Test invalid encoding
        let result = CompactArrayUtils::decode_length(&[0x80]);
        assert!(result.is_err());
    }
}