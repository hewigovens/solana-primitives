use crate::error::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// A 64-byte signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct SignatureBytes([u8; 64]);

impl Default for SignatureBytes {
    fn default() -> Self {
        Self([0; 64])
    }
}

impl SignatureBytes {
    /// Create a new signature from bytes
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Create a signature from a base58 string
    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s).into_vec().map_err(|_| {
            SolanaError::InvalidSignature(format!("failed to decode base58: {}", s))
        })?;
        if bytes.len() != 64 {
            return Err(SolanaError::InvalidSignature(format!(
                "invalid length: {}, expected: 64",
                bytes.len()
            )));
        }
        let mut result = [0; 64];
        result.copy_from_slice(&bytes);
        Ok(Self(result))
    }

    /// Convert the signature to a base58 string
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    /// Get the bytes of the signature
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl Serialize for SignatureBytes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_base58())
    }
}

impl<'de> Deserialize<'de> for SignatureBytes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        Self::from_base58(&s).map_err(serde::de::Error::custom)
    }
}
