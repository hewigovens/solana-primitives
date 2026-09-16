use crate::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A Solana public key (32 bytes).
///
/// Ordering is lexicographic over the bytes, matching the Solana SDK.
#[derive(
    Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, BorshSerialize, BorshDeserialize,
)]
pub struct Pubkey([u8; 32]);

impl Pubkey {
    /// Create a pubkey from raw bytes.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Parse a base58 literal at compile time.
    ///
    /// # Panics
    ///
    /// Panics (a compile error in const context) if `s` is not base58 for exactly 32 bytes.
    pub const fn from_str_const(s: &str) -> Self {
        let input = s.as_bytes();
        // Decoding succeeds into any buffer at least as long as the value, so
        // requiring a 31-byte decode to fail pins the length to exactly 32.
        assert!(
            bs58::decode(input).into_array_const::<31>().is_err(),
            "pubkey literal decodes to fewer than 32 bytes"
        );
        Self(bs58::decode(input).into_array_const_unwrap())
    }

    /// Parse a base58-encoded pubkey.
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
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Copy out the raw bytes.
    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl FromStr for Pubkey {
    type Err = SolanaError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_base58(s)
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_base58())
    }
}

impl fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pubkey({self})")
    }
}

impl From<[u8; 32]> for Pubkey {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Pubkey> for [u8; 32] {
    fn from(pubkey: Pubkey) -> Self {
        pubkey.0
    }
}

impl TryFrom<&[u8]> for Pubkey {
    type Error = SolanaError;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        <[u8; 32]>::try_from(bytes)
            .map(Self)
            .map_err(|_| SolanaError::InvalidLength {
                expected: 32,
                actual: bytes.len(),
            })
    }
}

impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Serialize for Pubkey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_base58())
    }
}

impl<'de> Deserialize<'de> for Pubkey {
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

    const TOKEN: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    #[test]
    fn base58_roundtrip_and_errors() {
        let pubkey = Pubkey::from_base58(TOKEN).unwrap();
        assert_eq!(pubkey.to_string(), TOKEN);
        assert_eq!(TOKEN.parse::<Pubkey>().unwrap(), pubkey);
        assert_eq!(Pubkey::from_str_const(TOKEN), pubkey);
        assert_eq!(
            Pubkey::from_str_const("11111111111111111111111111111111"),
            Pubkey::default()
        );

        assert_eq!(Pubkey::from_base58("0OIl"), Err(SolanaError::InvalidBase58));
        assert_eq!(
            Pubkey::from_base58("2g"),
            Err(SolanaError::InvalidLength {
                expected: 32,
                actual: 1
            })
        );
    }

    #[test]
    fn byte_conversions() {
        let bytes = [7u8; 32];
        let pubkey = Pubkey::from(bytes);
        assert_eq!(<[u8; 32]>::from(pubkey), bytes);
        assert_eq!(Pubkey::try_from(&bytes[..]).unwrap(), pubkey);
        assert_eq!(pubkey.as_ref(), &bytes[..]);
        assert_eq!(
            Pubkey::try_from(&bytes[..31]),
            Err(SolanaError::InvalidLength {
                expected: 32,
                actual: 31
            })
        );
    }

    #[test]
    fn orders_by_bytes() {
        let mut keys = [
            Pubkey::new([2; 32]),
            Pubkey::new([0; 32]),
            Pubkey::new([1; 32]),
        ];
        keys.sort();
        assert_eq!(
            keys,
            [
                Pubkey::new([0; 32]),
                Pubkey::new([1; 32]),
                Pubkey::new([2; 32])
            ]
        );
    }
}
