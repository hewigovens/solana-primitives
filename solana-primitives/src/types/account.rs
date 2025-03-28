use crate::types::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

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
}
