use crate::types::Pubkey;
use crate::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

const LOOKUP_TABLE_META_SIZE: usize = 56;

/// Address lookup table lookup information
/// Used to describe which addresses in a lookup table to use in a transaction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MessageAddressTableLookup {
    /// Address lookup table account key
    pub account_key: Pubkey,
    /// List of indices used to load writable account addresses
    #[serde(with = "crate::short_vec")]
    pub writable_indexes: Vec<u8>,
    /// List of indices used to load readonly account addresses
    #[serde(with = "crate::short_vec")]
    pub readonly_indexes: Vec<u8>,
}

impl MessageAddressTableLookup {
    /// Create a new MessageAddressTableLookup
    pub fn new(account_key: Pubkey, writable_indexes: Vec<u8>, readonly_indexes: Vec<u8>) -> Self {
        Self {
            account_key,
            writable_indexes,
            readonly_indexes,
        }
    }
}

/// Address lookup table account
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct AddressLookupTableAccount {
    /// The lookup table's public key
    pub key: Pubkey,
    /// List of addresses in the lookup table
    pub addresses: Vec<Pubkey>,
}

impl AddressLookupTableAccount {
    /// Create a new address lookup table account
    pub fn new(key: Pubkey, addresses: Vec<Pubkey>) -> Self {
        Self { key, addresses }
    }

    /// Get the number of addresses in the lookup table
    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    /// Check if the lookup table is empty
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }

    /// Get an address at the specified index
    pub fn get(&self, index: usize) -> Option<&Pubkey> {
        self.addresses.get(index)
    }

    /// Parse an address lookup table account from raw account data.
    pub fn from_account_data(key: Pubkey, data: &[u8]) -> Result<Self> {
        if data.len() < LOOKUP_TABLE_META_SIZE {
            return Err(SolanaError::InvalidMessage);
        }

        let address_data = &data[LOOKUP_TABLE_META_SIZE..];
        if !address_data.len().is_multiple_of(32) {
            return Err(SolanaError::InvalidMessage);
        }

        let mut addresses = Vec::with_capacity(address_data.len() / 32);
        for chunk in address_data.chunks_exact(32) {
            let bytes: [u8; 32] = chunk.try_into().map_err(|_| SolanaError::InvalidMessage)?;
            addresses.push(Pubkey::new(bytes));
        }

        Ok(Self { key, addresses })
    }

    /// Parse an address lookup table account from a base58 key and raw account data.
    pub fn from_base58_account_data(key: &str, data: &[u8]) -> Result<Self> {
        let key = Pubkey::from_base58(key)?;
        Self::from_account_data(key, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Pubkey;

    #[test]
    fn test_address_lookup_table_account() {
        let key = Pubkey::new([1; 32]);
        let addresses = vec![
            Pubkey::new([2; 32]),
            Pubkey::new([3; 32]),
            Pubkey::new([4; 32]),
        ];
        let lookup_table = AddressLookupTableAccount::new(key, addresses);

        // Test initial state
        assert_eq!(lookup_table.key, Pubkey::new([1; 32]));
        assert_eq!(lookup_table.len(), 3);
        assert!(!lookup_table.is_empty());

        // Test getting addresses
        assert_eq!(lookup_table.get(0), Some(&Pubkey::new([2; 32])));
        assert_eq!(lookup_table.get(1), Some(&Pubkey::new([3; 32])));
        assert_eq!(lookup_table.get(2), Some(&Pubkey::new([4; 32])));
        assert_eq!(lookup_table.get(3), None);
    }

    #[test]
    fn test_message_address_table_lookup() {
        let key = Pubkey::new([1; 32]);
        let writable_indexes = vec![0, 1];
        let readonly_indexes = vec![2];

        let lookup = MessageAddressTableLookup::new(key, writable_indexes, readonly_indexes);

        assert_eq!(lookup.account_key, Pubkey::new([1; 32]));
        assert_eq!(lookup.writable_indexes, vec![0, 1]);
        assert_eq!(lookup.readonly_indexes, vec![2]);
    }

    #[test]
    fn test_address_lookup_table_from_account_data() {
        let key = Pubkey::new([9; 32]);
        let mut data = vec![0u8; LOOKUP_TABLE_META_SIZE];
        data.extend_from_slice(&[2u8; 32]);
        data.extend_from_slice(&[3u8; 32]);

        let parsed = AddressLookupTableAccount::from_account_data(key, &data).unwrap();

        assert_eq!(parsed.key, key);
        assert_eq!(parsed.addresses.len(), 2);
        assert_eq!(parsed.addresses[0], Pubkey::new([2u8; 32]));
        assert_eq!(parsed.addresses[1], Pubkey::new([3u8; 32]));
    }

    #[test]
    fn test_address_lookup_table_from_account_data_rejects_invalid_length() {
        let key = Pubkey::new([9; 32]);
        let invalid_data = vec![0u8; LOOKUP_TABLE_META_SIZE + 1];

        let result = AddressLookupTableAccount::from_account_data(key, &invalid_data);

        assert!(matches!(result, Err(SolanaError::InvalidMessage)));
    }
}
