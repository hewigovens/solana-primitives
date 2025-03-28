use crate::types::{CompiledInstruction, MessageAddressTableLookup, Pubkey};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// The message header, identifying signed and read-only `account_keys`.
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MessageHeader {
    /// The number of signatures required for this message to be considered valid.
    pub num_required_signatures: u8,
    /// The last `num_readonly_signed_accounts` of the signed keys are read-only accounts.
    pub num_readonly_signed_accounts: u8,
    /// The last `num_readonly_unsigned_accounts` of the unsigned keys are read-only accounts.
    pub num_readonly_unsigned_accounts: u8,
}

/// Legacy message format (pre-versioned transactions)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct LegacyMessage {
    /// The message header, identifying signed and read-only `account_keys`.
    pub header: MessageHeader,
    /// List of account public keys
    pub account_keys: Vec<Pubkey>,
    /// The blockhash of a recent block.
    pub recent_blockhash: [u8; 32],
    /// Instructions that will be executed in sequence and committed in one atomic transaction if all succeed.
    pub instructions: Vec<CompiledInstruction>,
}

/// Versioned message format V0
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct VersionedMessageV0 {
    /// The message header, identifying signed and read-only `account_keys`.
    pub header: MessageHeader,
    /// List of account public keys
    pub account_keys: Vec<Pubkey>,
    /// The blockhash of a recent block.
    pub recent_blockhash: [u8; 32],
    /// Instructions that will be executed in sequence and committed in one atomic transaction if all succeed.
    pub instructions: Vec<CompiledInstruction>,
    /// List of address lookup table references
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

/// Versioned message format
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum VersionedMessage {
    /// Legacy message format (pre-versioned transactions)
    Legacy(LegacyMessage),
    /// Versioned message format V0
    V0(VersionedMessageV0),
}

/// A Solana transaction message
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Message {
    /// The message header, identifying signed and read-only `account_keys`.
    pub header: MessageHeader,
    /// List of account public keys
    pub account_keys: Vec<Pubkey>,
    /// The blockhash of a recent block.
    pub recent_blockhash: [u8; 32],
    /// Instructions that will be executed in sequence and committed in one atomic transaction if all succeed.
    pub instructions: Vec<CompiledInstruction>,
}

impl Message {
    /// Create a new message
    pub fn new(
        header: MessageHeader,
        account_keys: Vec<Pubkey>,
        recent_blockhash: [u8; 32],
        instructions: Vec<CompiledInstruction>,
    ) -> Self {
        Self {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        }
    }

    /// Get the number of required signatures
    pub fn num_required_signatures(&self) -> u8 {
        self.header.num_required_signatures
    }

    /// Get the number of read-only signed accounts
    pub fn num_readonly_signed_accounts(&self) -> u8 {
        self.header.num_readonly_signed_accounts
    }

    /// Get the number of read-only unsigned accounts
    pub fn num_readonly_unsigned_accounts(&self) -> u8 {
        self.header.num_readonly_unsigned_accounts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CompiledInstruction, Pubkey};

    #[test]
    fn test_message() {
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

        let message = Message::new(header, account_keys, recent_blockhash, instructions);

        assert_eq!(message.num_required_signatures(), 1);
        assert_eq!(message.num_readonly_signed_accounts(), 0);
        assert_eq!(message.num_readonly_unsigned_accounts(), 1);
    }

    #[test]
    fn test_versioned_message() {
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

        // Create a V0 message with address table lookups
        let address_table_lookups = vec![MessageAddressTableLookup::new(
            Pubkey::new([2; 32]),
            vec![0, 1], // writable indexes
            vec![2],    // readonly indexes
        )];

        let v0_message = VersionedMessageV0 {
            header: header.clone(),
            account_keys: account_keys.clone(),
            recent_blockhash,
            instructions: instructions.clone(),
            address_table_lookups,
        };

        // Create a versioned message
        let versioned_message = VersionedMessage::V0(v0_message);

        // Verify the contents
        match versioned_message {
            VersionedMessage::Legacy(_) => panic!("Expected V0 message"),
            VersionedMessage::V0(msg) => {
                assert_eq!(msg.header.num_required_signatures, 1);
                assert_eq!(msg.account_keys.len(), 2);
                assert_eq!(msg.instructions.len(), 1);
                assert_eq!(msg.address_table_lookups.len(), 1);
                assert_eq!(
                    msg.address_table_lookups[0].account_key,
                    Pubkey::new([2; 32])
                );
                assert_eq!(msg.address_table_lookups[0].writable_indexes, vec![0, 1]);
                assert_eq!(msg.address_table_lookups[0].readonly_indexes, vec![2]);
            }
        }
    }
}
