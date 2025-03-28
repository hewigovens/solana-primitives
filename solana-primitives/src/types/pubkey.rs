use crate::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// A Solana public key (32 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct Pubkey([u8; 32]);

impl FromStr for Pubkey {
    type Err = SolanaError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_base58(s)
    }
}

impl Ord for Pubkey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Pubkey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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

impl Pubkey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| SolanaError::InvalidPubkeyLength)?;

        if bytes.len() != 32 {
            return Err(SolanaError::InvalidPubkeyLength);
        }

        Ok(Self(bytes.try_into().unwrap()))
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
