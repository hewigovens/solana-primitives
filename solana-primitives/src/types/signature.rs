use crate::error::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A 64-byte Ed25519 signature.
///
/// The all-zero value is the placeholder for a missing signature.
#[derive(Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct SignatureBytes([u8; 64]);

impl Default for SignatureBytes {
    fn default() -> Self {
        Self([0; 64])
    }
}

impl SignatureBytes {
    /// Create a signature from raw bytes.
    pub const fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Parse a base58-encoded signature.
    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| SolanaError::InvalidBase58)?;
        Self::try_from(bytes.as_slice())
    }

    /// Encode as base58.
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    /// Borrow the raw bytes.
    pub const fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Whether this is the all-zero placeholder.
    pub fn is_placeholder(&self) -> bool {
        self.0 == [0; 64]
    }
}

impl FromStr for SignatureBytes {
    type Err = SolanaError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_base58(s)
    }
}

impl fmt::Display for SignatureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_base58())
    }
}

impl fmt::Debug for SignatureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignatureBytes({self})")
    }
}

impl From<[u8; 64]> for SignatureBytes {
    fn from(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

impl From<SignatureBytes> for [u8; 64] {
    fn from(signature: SignatureBytes) -> Self {
        signature.0
    }
}

impl TryFrom<&[u8]> for SignatureBytes {
    type Error = SolanaError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        <[u8; 64]>::try_from(bytes)
            .map(Self)
            .map_err(|_| SolanaError::InvalidLength {
                expected: 64,
                actual: bytes.len(),
            })
    }
}

impl AsRef<[u8]> for SignatureBytes {
    fn as_ref(&self) -> &[u8] {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions() {
        let signature = SignatureBytes::from([9u8; 64]);
        assert_eq!(
            SignatureBytes::from_base58(&signature.to_base58()).unwrap(),
            signature
        );
        assert_eq!(SignatureBytes::try_from(&[9u8; 64][..]).unwrap(), signature);
        assert_eq!(
            SignatureBytes::try_from(&[9u8; 32][..]),
            Err(SolanaError::InvalidLength {
                expected: 64,
                actual: 32
            })
        );
        assert!(SignatureBytes::default().is_placeholder());
        assert!(!signature.is_placeholder());
    }
}
