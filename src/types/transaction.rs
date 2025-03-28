use crate::types::{
    CompiledInstruction, LegacyMessage, Message, Pubkey, SignatureBytes, VersionedMessage,
    VersionedMessageV0,
};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// A Solana transaction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Transaction {
    /// The signatures
    pub signatures: Vec<SignatureBytes>,
    /// The message
    pub message: Message,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(message: Message) -> Self {
        Self {
            signatures: Vec::new(),
            message,
        }
    }

    /// Add a signature to the transaction
    pub fn add_signature(&mut self, signature: SignatureBytes) {
        self.signatures.push(signature);
    }

    /// Get the number of required signatures
    pub fn num_required_signatures(&self) -> u8 {
        self.message.num_required_signatures()
    }

    /// Get the number of read-only signed accounts
    pub fn num_readonly_signed_accounts(&self) -> u8 {
        self.message.num_readonly_signed_accounts()
    }

    /// Get the number of read-only unsigned accounts
    pub fn num_readonly_unsigned_accounts(&self) -> u8 {
        self.message.num_readonly_unsigned_accounts()
    }

    /// Get the account keys
    pub fn account_keys(&self) -> &[Pubkey] {
        &self.message.account_keys
    }

    /// Get the recent blockhash
    pub fn recent_blockhash(&self) -> &[u8; 32] {
        &self.message.recent_blockhash
    }

    /// Get the instructions
    pub fn instructions(&self) -> &[CompiledInstruction] {
        &self.message.instructions
    }

    /// Deserialize a transaction from bytes
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.is_empty() {
            return Err("Empty transaction data".into());
        }

        // First byte is the signature count
        let num_signatures = bytes[0] as usize;

        // Check if there are enough bytes for signatures
        if bytes.len() < 1 + (num_signatures * 64) {
            return Err("Not enough bytes for signatures".into());
        }

        // Extract signatures
        let mut signatures = Vec::with_capacity(num_signatures);
        let mut offset = 1; // Skip signature count byte

        for _ in 0..num_signatures {
            if offset + 64 > bytes.len() {
                return Err("Invalid signature data".into());
            }

            let sig_bytes: [u8; 64] = bytes[offset..offset + 64]
                .try_into()
                .map_err(|_| "Failed to convert signature bytes")?;

            signatures.push(SignatureBytes::new(sig_bytes));
            offset += 64;
        }

        // The rest is the message
        let message_bytes = &bytes[offset..];

        // Use our manual decoder to decode the legacy message
        match manual_decode::decode_legacy_message(message_bytes, Vec::new()) {
            Ok(VersionedTransaction::Legacy { message, .. }) => {
                // We know this is a legacy message, convert to regular Message
                let regular_message = Message {
                    header: message.header,
                    account_keys: message.account_keys,
                    recent_blockhash: message.recent_blockhash,
                    instructions: message.instructions,
                };

                Ok(Self {
                    signatures,
                    message: regular_message,
                })
            }
            _ => Err("Failed to decode legacy message for Transaction".into()),
        }
    }
}

/// Versioned transaction format
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum VersionedTransaction {
    /// Legacy transaction format (pre-versioned transactions)
    Legacy {
        /// List of signatures
        signatures: Vec<SignatureBytes>,
        /// Message to sign
        message: LegacyMessage,
    },
    /// Versioned transaction format V0
    V0 {
        /// List of signatures
        signatures: Vec<SignatureBytes>,
        /// Message to sign
        message: VersionedMessageV0,
    },
}

impl VersionedTransaction {
    /// Create a new versioned transaction
    pub fn new(message: VersionedMessage) -> Self {
        match message {
            VersionedMessage::Legacy(msg) => Self::Legacy {
                signatures: Vec::new(),
                message: msg,
            },
            VersionedMessage::V0(msg) => Self::V0 {
                signatures: Vec::new(),
                message: msg,
            },
        }
    }

    /// Add a signature to the transaction
    pub fn add_signature(&mut self, signature: SignatureBytes) {
        match self {
            Self::Legacy { signatures, .. } => signatures.push(signature),
            Self::V0 { signatures, .. } => signatures.push(signature),
        }
    }

    /// Get the number of required signatures
    pub fn num_required_signatures(&self) -> u8 {
        match self {
            Self::Legacy { message, .. } => message.header.num_required_signatures,
            Self::V0 { message, .. } => message.header.num_required_signatures,
        }
    }

    /// Get the number of read-only signed accounts
    pub fn num_readonly_signed_accounts(&self) -> u8 {
        match self {
            Self::Legacy { message, .. } => message.header.num_readonly_signed_accounts,
            Self::V0 { message, .. } => message.header.num_readonly_signed_accounts,
        }
    }

    /// Get the number of read-only unsigned accounts
    pub fn num_readonly_unsigned_accounts(&self) -> u8 {
        match self {
            Self::Legacy { message, .. } => message.header.num_readonly_unsigned_accounts,
            Self::V0 { message, .. } => message.header.num_readonly_unsigned_accounts,
        }
    }

    /// Get the account keys
    pub fn account_keys(&self) -> &[Pubkey] {
        match self {
            Self::Legacy { message, .. } => &message.account_keys,
            Self::V0 { message, .. } => &message.account_keys,
        }
    }

    /// Get the recent blockhash
    pub fn recent_blockhash(&self) -> &[u8; 32] {
        match self {
            Self::Legacy { message, .. } => &message.recent_blockhash,
            Self::V0 { message, .. } => &message.recent_blockhash,
        }
    }

    /// Get the instructions
    pub fn instructions(&self) -> &[CompiledInstruction] {
        match self {
            Self::Legacy { message, .. } => &message.instructions,
            Self::V0 { message, .. } => &message.instructions,
        }
    }

    /// Deserialize a versioned transaction from bytes
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.is_empty() {
            return Err("Empty transaction data".into());
        }

        // First byte is the signature count
        let num_signatures = bytes[0] as usize;

        // Check if there are enough bytes for signatures
        if bytes.len() < 1 + (num_signatures * 64) {
            return Err("Not enough bytes for signatures".into());
        }

        // Extract signatures
        let mut signatures = Vec::with_capacity(num_signatures);
        let mut offset = 1; // Skip signature count byte

        for _ in 0..num_signatures {
            if offset + 64 > bytes.len() {
                return Err("Invalid signature data".into());
            }

            let sig_bytes: [u8; 64] = bytes[offset..offset + 64]
                .try_into()
                .map_err(|_| "Failed to convert signature bytes")?;

            signatures.push(SignatureBytes::new(sig_bytes));
            offset += 64;
        }

        // The rest is the message
        let message_bytes = &bytes[offset..];

        // Manually decode the message
        self::manual_decode::decode_message(message_bytes, signatures)
    }
}

/// Module for manual decoding of Solana message format
mod manual_decode {
    use super::*;
    use crate::types::MessageHeader;

    /// Decode a message based on the Solana binary format
    /// The format is:
    /// 1. If the high bit of the first byte is set, it's a versioned message
    ///    - The version is in the lower 7 bits
    ///    - Rest of message follows based on version
    /// 2. Otherwise, it's a legacy message with format:
    ///    - 3 bytes header (num_required_signatures, num_readonly_signed, num_readonly_unsigned)
    ///    - Account keys (1 byte count, then count * 32 bytes)
    ///    - Recent blockhash (32 bytes)
    ///    - Instructions (1 byte count, then variable length instructions)
    pub fn decode_message(
        bytes: &[u8],
        signatures: Vec<SignatureBytes>,
    ) -> Result<VersionedTransaction, Box<dyn std::error::Error>> {
        if bytes.len() < 3 {
            return Err("Message bytes too short, need at least 3 bytes for header".into());
        }

        // Check if this is a versioned message (first bit set)
        let is_versioned = (bytes[0] & 0x80) != 0;

        if is_versioned {
            // Extract version from first byte (low 7 bits)
            let version = bytes[0] & 0x7F;

            // Currently only V0 messages are supported
            if version == 0 {
                decode_v0_message(&bytes[1..], signatures)
            } else {
                Err(format!("Unsupported message version: {}", version).into())
            }
        } else {
            // Legacy message (no version byte)
            decode_legacy_message(bytes, signatures)
        }
    }

    /// Decode a legacy (non-versioned) message
    /// The format is:
    /// 1. Header (3 bytes)
    ///    - num_required_signatures (1 byte)
    ///    - num_readonly_signed_accounts (1 byte)
    ///    - num_readonly_unsigned_accounts (1 byte)
    /// 2. Account keys
    ///    - count (1 byte)
    ///    - public keys (count * 32 bytes)
    /// 3. Recent blockhash (32 bytes)
    /// 4. Instructions
    ///    - count (1 byte)
    ///    - instructions (variable length)
    pub fn decode_legacy_message(
        bytes: &[u8],
        signatures: Vec<SignatureBytes>,
    ) -> Result<VersionedTransaction, Box<dyn std::error::Error>> {
        if bytes.len() < 3 {
            return Err("Legacy message too short".into());
        }

        // Header: 3 bytes
        let header = MessageHeader {
            num_required_signatures: bytes[0],
            num_readonly_signed_accounts: bytes[1],
            num_readonly_unsigned_accounts: bytes[2],
        };

        let mut offset = 3;

        // Account keys
        if offset >= bytes.len() {
            return Err("Message too short: no account count".into());
        }
        let account_count = bytes[offset] as usize;
        offset += 1;

        if offset + (account_count * 32) > bytes.len() {
            return Err("Message too short: not enough bytes for accounts".into());
        }

        let mut account_keys = Vec::with_capacity(account_count);
        for _ in 0..account_count {
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes[offset..offset + 32]);
            account_keys.push(Pubkey::new(key));
            offset += 32;
        }

        // Recent blockhash (always 32 bytes)
        if offset + 32 > bytes.len() {
            return Err("Message too short: no recent blockhash".into());
        }
        let mut recent_blockhash = [0u8; 32];
        recent_blockhash.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Instructions
        if offset >= bytes.len() {
            return Err("Message too short: no instruction count".into());
        }
        let instruction_count = bytes[offset] as usize;
        offset += 1;

        let mut instructions = Vec::with_capacity(instruction_count);
        for _ in 0..instruction_count {
            if offset >= bytes.len() {
                return Err("Message too short: incomplete instruction".into());
            }

            // Program ID index (1 byte)
            let program_id_index = bytes[offset];
            offset += 1;

            if offset >= bytes.len() {
                return Err("Message too short: no account indices count".into());
            }

            // Account indices (1 byte count, then count bytes)
            let account_indices_count = bytes[offset] as usize;
            offset += 1;

            if offset + account_indices_count > bytes.len() {
                return Err("Message too short: not enough account indices".into());
            }

            let accounts = bytes[offset..offset + account_indices_count].to_vec();
            offset += account_indices_count;

            if offset >= bytes.len() {
                return Err("Message too short: no instruction data length".into());
            }

            // Instruction data (1 byte length, then length bytes)
            let data_length = bytes[offset] as usize;
            offset += 1;

            if offset + data_length > bytes.len() {
                return Err("Message too short: not enough instruction data".into());
            }

            let data = bytes[offset..offset + data_length].to_vec();
            offset += data_length;

            instructions.push(CompiledInstruction {
                program_id_index,
                accounts,
                data,
            });
        }

        Ok(VersionedTransaction::Legacy {
            signatures,
            message: LegacyMessage {
                header,
                account_keys,
                recent_blockhash,
                instructions,
            },
        })
    }

    /// Decode a V0 versioned message
    /// Currently this is similar to legacy but with a version byte at the start
    /// V0 also supports address lookup tables, but we're not handling that for simplicity
    pub fn decode_v0_message(
        bytes: &[u8],
        signatures: Vec<SignatureBytes>,
    ) -> Result<VersionedTransaction, Box<dyn std::error::Error>> {
        // V0 message format is almost identical to legacy except for potential address lookup tables
        // For simplicity, we'll reuse the legacy message decoding logic and then convert

        // First, decode as if it were a legacy message
        let legacy_result = decode_legacy_message(bytes, Vec::new())?;

        // Extract the message
        if let VersionedTransaction::Legacy { message, .. } = legacy_result {
            // Convert to V0 format
            Ok(VersionedTransaction::V0 {
                signatures,
                message: VersionedMessageV0 {
                    header: message.header,
                    account_keys: message.account_keys,
                    recent_blockhash: message.recent_blockhash,
                    instructions: message.instructions,
                },
            })
        } else {
            Err("Unexpected error in message conversion".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{CompiledInstruction, MessageHeader, Pubkey, SignatureBytes},
        VersionedMessageV0,
    };

    fn create_test_message() -> Message {
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

        Message::new(header, account_keys, recent_blockhash, instructions)
    }

    #[test]
    fn test_transaction() {
        let message = create_test_message();
        let mut transaction = Transaction::new(message);

        // Test initial state
        assert_eq!(transaction.signatures.len(), 0);
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.num_readonly_signed_accounts(), 0);
        assert_eq!(transaction.num_readonly_unsigned_accounts(), 1);

        // Test adding signature
        let signature = SignatureBytes::new([0; 64]);
        transaction.add_signature(signature);
        assert_eq!(transaction.signatures.len(), 1);
    }

    #[test]
    fn test_versioned_transaction() {
        let message = VersionedMessage::V0(VersionedMessageV0 {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])],
            recent_blockhash: [0u8; 32],
            instructions: vec![CompiledInstruction {
                program_id_index: 1,
                accounts: vec![0],
                data: vec![],
            }],
        });
        let mut transaction = VersionedTransaction::new(message);

        // Test initial state
        match &transaction {
            VersionedTransaction::V0 { signatures, .. } => {
                assert_eq!(signatures.len(), 0);
            }
            _ => panic!("Expected V0 transaction"),
        }
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.num_readonly_signed_accounts(), 0);
        assert_eq!(transaction.num_readonly_unsigned_accounts(), 1);

        // Test adding signature
        let signature = SignatureBytes::new([0; 64]);
        transaction.add_signature(signature);
        match &transaction {
            VersionedTransaction::V0 { signatures, .. } => {
                assert_eq!(signatures.len(), 1);
            }
            _ => panic!("Expected V0 transaction"),
        }
    }
}
