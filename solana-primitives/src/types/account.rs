use crate::types::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Address lookup table lookup information
/// Used to describe which addresses in a lookup table to use in a transaction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MessageAddressTableLookup {
    /// Address lookup table account key
    pub account_key: Pubkey,
    /// List of indices used to load writable account addresses
    #[serde(with = "crate::utils::short_vec")]
    pub writable_indexes: Vec<u8>,
    /// List of indices used to load readonly account addresses
    #[serde(with = "crate::utils::short_vec")]
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
}
