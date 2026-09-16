use crate::Result;
use crate::crypto::sign_message;
use crate::error::SolanaError;
use crate::instructions::program_ids::COMPUTE_BUDGET_PROGRAM_ID;
use crate::types::{
    CompiledInstruction, Instruction, LegacyMessage, MAX_TRANSACTION_SIZE, Message,
    MessageAddressTableLookup, Pubkey, SignatureBytes, VersionedMessage, VersionedMessageV0,
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

    /// Deserialize a legacy transaction from wire bytes.
    ///
    /// Versioned (v0) transactions are rejected; use [`VersionedTransaction`] for those.
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self> {
        match VersionedTransaction::deserialize_with_version(bytes)? {
            VersionedTransaction::Legacy {
                signatures,
                message,
            } => Ok(Self {
                signatures,
                message: Message {
                    header: message.header,
                    account_keys: message.account_keys,
                    recent_blockhash: message.recent_blockhash,
                    instructions: message.instructions,
                },
            }),
            VersionedTransaction::V0 { .. } => Err(SolanaError::DeserializationError(
                "expected a legacy transaction".to_string(),
            )),
        }
    }

    /// Serializes the full transaction into the Solana legacy wire format.
    pub fn serialize_legacy(&self) -> Result<Vec<u8>> {
        let mut tx_wire_bytes: Vec<u8> = Vec::new();

        // 1. Number of signatures (Compact-U16 encoded)
        let sig_len_bytes = crate::encode_length_to_compact_u16_bytes(self.signatures.len())?;
        tx_wire_bytes.extend_from_slice(&sig_len_bytes);

        // 2. Signatures
        for sig_bytes_wrapper in &self.signatures {
            tx_wire_bytes.extend_from_slice(sig_bytes_wrapper.as_bytes());
        }

        // 3. Serialized Message
        // The `serialize_for_signing` method in `Message` returns Result<Vec<u8>, String>
        let serialized_message = self
            .message
            .serialize_for_signing()
            .map_err(SolanaError::SerializationError)?;
        tx_wire_bytes.extend_from_slice(&serialized_message);

        Ok(tx_wire_bytes)
    }

    /// Sign the transaction with one or more private keys
    /// The private keys must correspond to the signing accounts in the same order
    pub fn sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        // Get message bytes for signing
        let message_bytes = self
            .message
            .serialize_for_signing()
            .map_err(SolanaError::SerializationError)?;

        // Clear existing signatures
        self.signatures.clear();

        // Get number of required signatures
        let num_required_sigs = self.message.header.num_required_signatures as usize;

        // Validate we have enough private keys
        if private_keys.len() < num_required_sigs {
            return Err(SolanaError::InvalidSignature(format!(
                "insufficient private keys: {}, required: {}",
                private_keys.len(),
                num_required_sigs
            )));
        }

        // Sign with each private key
        for private_key in private_keys.iter().take(num_required_sigs) {
            let signature = sign_message(private_key, &message_bytes)?;
            self.signatures.push(signature);
        }

        Ok(())
    }

    /// Partially sign the transaction with specific private keys
    /// Updates only the signatures for the provided keys based on their public key positions
    pub fn partial_sign(&mut self, private_keys: &[&[u8]], public_keys: &[Pubkey]) -> Result<()> {
        if private_keys.len() != public_keys.len() {
            return Err(SolanaError::InvalidSignature(format!(
                "private keys count ({}) does not match public keys count ({})",
                private_keys.len(),
                public_keys.len()
            )));
        }

        // Get message bytes for signing
        let message_bytes = self
            .message
            .serialize_for_signing()
            .map_err(SolanaError::SerializationError)?;

        // Ensure we have enough signature slots
        let num_required_sigs = self.message.header.num_required_signatures as usize;
        if self.signatures.len() < num_required_sigs {
            self.signatures
                .resize(num_required_sigs, SignatureBytes::new([0u8; 64]));
        }

        // Sign with each private key and place signature at correct index
        for (private_key, public_key) in private_keys.iter().zip(public_keys.iter()) {
            // Find the index of this public key in account_keys
            if let Some(index) = self
                .message
                .account_keys
                .iter()
                .position(|k| k == public_key)
                && index < num_required_sigs
            {
                let signature = sign_message(private_key, &message_bytes)?;
                self.signatures[index] = signature;
            }
        }

        Ok(())
    }

    /// Check if the transaction has been signed by all required signers
    pub fn is_signed(&self) -> bool {
        let num_required = self.message.header.num_required_signatures as usize;
        if self.signatures.len() < num_required {
            return false;
        }

        // Check that none of the required signatures are empty
        for i in 0..num_required {
            if self.signatures[i].as_bytes().iter().all(|&b| b == 0) {
                return false;
            }
        }

        true
    }

    /// Validate transaction size is within limits (1232 bytes)
    pub fn validate_size(&self) -> Result<()> {
        let serialized = self.serialize_legacy()?;

        if serialized.len() > MAX_TRANSACTION_SIZE {
            return Err(SolanaError::SerializationError(format!(
                "Transaction size {} exceeds maximum of {} bytes",
                serialized.len(),
                MAX_TRANSACTION_SIZE
            )));
        }

        Ok(())
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

    pub fn signatures(&self) -> &[SignatureBytes] {
        match self {
            Self::Legacy { signatures, .. } => signatures,
            Self::V0 { signatures, .. } => signatures,
        }
    }

    pub fn signatures_mut(&mut self) -> &mut Vec<SignatureBytes> {
        match self {
            Self::Legacy { signatures, .. } => signatures,
            Self::V0 { signatures, .. } => signatures,
        }
    }

    pub fn instructions_mut(&mut self) -> &mut Vec<CompiledInstruction> {
        match self {
            Self::Legacy { message, .. } => &mut message.instructions,
            Self::V0 { message, .. } => &mut message.instructions,
        }
    }

    fn compute_budget_program_index(&self) -> Option<u8> {
        let cb_pubkey = Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).ok()?;
        self.account_keys()
            .iter()
            .position(|k| *k == cb_pubkey)
            .map(|i| i as u8)
    }

    pub fn get_compute_unit_price(&self) -> Option<u64> {
        let idx = self.compute_budget_program_index()?;
        for ix in self.instructions() {
            if ix.program_id_index == idx && ix.data.len() == 9 && ix.data[0] == 3 {
                return Some(u64::from_le_bytes(ix.data[1..9].try_into().ok()?));
            }
        }
        None
    }

    pub fn set_compute_unit_price(&mut self, micro_lamports: u64) -> Result<bool> {
        if let Some(idx) = self.compute_budget_program_index() {
            for ix in self.instructions_mut() {
                if ix.program_id_index == idx && ix.data.len() == 9 && ix.data[0] == 3 {
                    ix.data[1..9].copy_from_slice(&micro_lamports.to_le_bytes());
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn get_compute_unit_limit(&self) -> Option<u32> {
        let idx = self.compute_budget_program_index()?;
        for ix in self.instructions() {
            if ix.program_id_index == idx && ix.data.len() == 5 && ix.data[0] == 2 {
                return Some(u32::from_le_bytes(ix.data[1..5].try_into().ok()?));
            }
        }
        None
    }

    pub fn set_compute_unit_limit(&mut self, units: u32) -> Result<bool> {
        if let Some(idx) = self.compute_budget_program_index() {
            for ix in self.instructions_mut() {
                if ix.program_id_index == idx && ix.data.len() == 5 && ix.data[0] == 2 {
                    ix.data[1..5].copy_from_slice(&units.to_le_bytes());
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> Result<()> {
        let message = match self {
            Self::Legacy { message, .. } => message,
            _ => {
                return Err(SolanaError::SerializationError(
                    "add_instruction only supported for legacy transactions".to_string(),
                ));
            }
        };

        let mut new_writable_non_signers: Vec<Pubkey> = Vec::new();
        let mut new_readonly_non_signers: Vec<Pubkey> = Vec::new();

        if !message.account_keys.contains(&instruction.program_id) {
            new_readonly_non_signers.push(instruction.program_id);
        }

        for meta in &instruction.accounts {
            if !message.account_keys.contains(&meta.pubkey)
                && !new_writable_non_signers.contains(&meta.pubkey)
                && !new_readonly_non_signers.contains(&meta.pubkey)
            {
                if meta.is_writable && !meta.is_signer {
                    new_writable_non_signers.push(meta.pubkey);
                } else if !meta.is_signer {
                    new_readonly_non_signers.push(meta.pubkey);
                }
            }
        }

        let insert_pos = message
            .account_keys
            .len()
            .checked_sub(message.header.num_readonly_unsigned_accounts as usize)
            .ok_or(SolanaError::InvalidMessage)?;
        for (i, pubkey) in new_writable_non_signers.iter().enumerate() {
            message.account_keys.insert(insert_pos + i, *pubkey);
        }

        let num_inserted = new_writable_non_signers.len();
        if num_inserted > 0 {
            for ix in &mut message.instructions {
                if (ix.program_id_index as usize) >= insert_pos {
                    ix.program_id_index += num_inserted as u8;
                }
                for acc in &mut ix.accounts {
                    if (*acc as usize) >= insert_pos {
                        *acc += num_inserted as u8;
                    }
                }
            }
        }

        for pubkey in &new_readonly_non_signers {
            message.account_keys.push(*pubkey);
        }
        message.header.num_readonly_unsigned_accounts += new_readonly_non_signers.len() as u8;

        let program_id_index = message
            .account_keys
            .iter()
            .position(|k| *k == instruction.program_id)
            .unwrap() as u8;
        let accounts: Vec<u8> = instruction
            .accounts
            .iter()
            .map(|meta| {
                message
                    .account_keys
                    .iter()
                    .position(|k| *k == meta.pubkey)
                    .unwrap() as u8
            })
            .collect();

        message.instructions.push(CompiledInstruction {
            program_id_index,
            accounts,
            data: instruction.data,
        });

        Ok(())
    }

    pub fn serialize_message(&self) -> Result<Vec<u8>> {
        match self {
            Self::Legacy { message, .. } => message
                .serialize_for_signing()
                .map_err(SolanaError::SerializationError),
            Self::V0 { message, .. } => message
                .serialize_for_signing()
                .map_err(SolanaError::SerializationError),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let signatures = self.signatures();
        let sig_len = crate::encode_length_to_compact_u16_bytes(signatures.len())
            .map_err(SolanaError::SerializationError)?;
        bytes.extend_from_slice(&sig_len);
        for sig in signatures {
            bytes.extend_from_slice(sig.as_bytes());
        }
        let message_bytes = self.serialize_message()?;
        bytes.extend_from_slice(&message_bytes);
        Ok(bytes)
    }

    /// Deserialize and sanitize a legacy or v0 transaction from wire bytes.
    ///
    /// The input must be exactly one transaction: truncated input, trailing bytes,
    /// non-canonical compact-u16 lengths, and messages that fail Solana's
    /// sanitization rules are all rejected.
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self> {
        // Legacy and v0 transactions start with a one-byte signature count (< 0x80);
        // a set high bit would be a versioned message prefix.
        let (&num_signatures, rest) = bytes.split_first().ok_or_else(|| {
            SolanaError::DeserializationError("Empty transaction data".to_string())
        })?;
        if num_signatures & 0x80 != 0 {
            return Err(SolanaError::DeserializationError(format!(
                "invalid transaction discriminator: {num_signatures:#04x}"
            )));
        }

        let signatures_len = num_signatures as usize * 64;
        if rest.len() < signatures_len {
            return Err(SolanaError::DeserializationError(
                "Not enough bytes for signatures".to_string(),
            ));
        }
        let (signature_bytes, message_bytes) = rest.split_at(signatures_len);
        let signatures = signature_bytes
            .as_chunks::<64>()
            .0
            .iter()
            .copied()
            .map(SignatureBytes::new)
            .collect();

        let transaction = manual_decode::decode_message(message_bytes, signatures)?;
        manual_decode::sanitize(&transaction)?;
        Ok(transaction)
    }
}

/// Module for manual decoding of Solana message format
mod manual_decode {
    use super::*;
    use crate::types::MessageHeader;

    fn sanitize_error(message: &str) -> SolanaError {
        SolanaError::DeserializationError(format!("invalid message: {message}"))
    }

    /// Solana's legacy/v0 message and signature sanitization rules.
    pub fn sanitize(transaction: &VersionedTransaction) -> Result<()> {
        let (header, account_keys, instructions, lookups) = match transaction {
            VersionedTransaction::Legacy { message, .. } => (
                &message.header,
                &message.account_keys,
                &message.instructions,
                &[][..],
            ),
            VersionedTransaction::V0 { message, .. } => (
                &message.header,
                &message.account_keys,
                &message.instructions,
                &message.address_table_lookups[..],
            ),
        };

        let num_static_keys = account_keys.len();
        let num_required_signatures = header.num_required_signatures as usize;
        if num_required_signatures + header.num_readonly_unsigned_accounts as usize
            > num_static_keys
        {
            return Err(sanitize_error(
                "header describes more accounts than account_keys",
            ));
        }
        // There must be at least one writable signer (the fee payer).
        if header.num_readonly_signed_accounts >= header.num_required_signatures {
            return Err(sanitize_error("no writable fee payer"));
        }

        let mut num_lookup_keys = 0usize;
        for lookup in lookups {
            let num_indexes = lookup.writable_indexes.len() + lookup.readonly_indexes.len();
            if num_indexes == 0 {
                return Err(sanitize_error("address table lookup loads no accounts"));
            }
            num_lookup_keys += num_indexes;
        }
        let num_keys = num_static_keys + num_lookup_keys;
        // Account indexes are u8.
        if num_keys > 256 {
            return Err(sanitize_error("more than 256 accounts"));
        }

        for instruction in instructions {
            // Programs must be static keys, and the fee payer can't be a program.
            let program_index = instruction.program_id_index as usize;
            if program_index == 0 || program_index >= num_static_keys {
                return Err(sanitize_error("invalid program id index"));
            }
            if instruction
                .accounts
                .iter()
                .any(|&index| index as usize >= num_keys)
            {
                return Err(sanitize_error("invalid account index"));
            }
        }

        if transaction.signatures().len() != num_required_signatures {
            return Err(SolanaError::DeserializationError(format!(
                "signature count {} does not match num_required_signatures {}",
                transaction.signatures().len(),
                num_required_signatures
            )));
        }
        Ok(())
    }

    fn ensure_consumed(bytes: &[u8], offset: usize) -> Result<()> {
        if offset != bytes.len() {
            return Err(SolanaError::DeserializationError(format!(
                "{} trailing bytes after message",
                bytes.len() - offset
            )));
        }
        Ok(())
    }

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
    ) -> Result<VersionedTransaction> {
        if bytes.len() < 3 {
            return Err(SolanaError::DeserializationError(
                "Message bytes too short, need at least 3 bytes for header".to_string(),
            ));
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
                Err(SolanaError::DeserializationError(format!(
                    "Unsupported message version: {version}"
                )))
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
    ) -> Result<VersionedTransaction> {
        if bytes.len() < 3 {
            return Err(SolanaError::DeserializationError(
                "Legacy message too short".to_string(),
            ));
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
            return Err(SolanaError::DeserializationError(
                "Message too short: no account count".to_string(),
            ));
        }
        let (account_count, len_bytes_consumed) =
            crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
        offset += len_bytes_consumed;

        if offset + (account_count * 32) > bytes.len() {
            return Err(SolanaError::DeserializationError(
                "Message too short: not enough bytes for accounts".to_string(),
            ));
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
            return Err(SolanaError::DeserializationError(
                "Message too short: no recent blockhash".to_string(),
            ));
        }
        let mut recent_blockhash = [0u8; 32];
        recent_blockhash.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Instructions
        if offset >= bytes.len() {
            return Err(SolanaError::DeserializationError(
                "Message too short: no instruction count".to_string(),
            ));
        }
        let (instruction_count, len_bytes_consumed) =
            crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
        offset += len_bytes_consumed;

        // Each instruction needs >= 3 bytes; reject counts that can't fit in what's left.
        let remaining = bytes.len().saturating_sub(offset);
        if instruction_count.saturating_mul(3) > remaining {
            return Err(SolanaError::DeserializationError(
                "Message too short: instruction count exceeds remaining bytes".to_string(),
            ));
        }

        let mut instructions = Vec::with_capacity(instruction_count);
        for _ in 0..instruction_count {
            if offset >= bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: incomplete instruction".to_string(),
                ));
            }

            // Program ID index (1 byte)
            let program_id_index = bytes[offset];
            offset += 1;

            if offset >= bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: no account indices count".to_string(),
                ));
            }

            // Account indices (compact-u16 length, then count bytes)
            let (account_indices_count, len_bytes_consumed) =
                crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
            offset += len_bytes_consumed;

            if offset + account_indices_count > bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: not enough account indices".to_string(),
                ));
            }

            let accounts = bytes[offset..offset + account_indices_count].to_vec();
            offset += account_indices_count;

            if offset >= bytes.len() {
                // This check ensures there's at least one byte for the length itself.
                return Err(SolanaError::DeserializationError(
                    "Message too short: no instruction data length".to_string(),
                ));
            }

            // Instruction data (compact-u16 length, then length bytes)
            let (data_length, len_bytes_consumed) =
                crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
            offset += len_bytes_consumed;

            if offset + data_length > bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: not enough instruction data".to_string(),
                ));
            }

            let data = bytes[offset..offset + data_length].to_vec();
            offset += data_length;

            instructions.push(CompiledInstruction {
                program_id_index,
                accounts,
                data,
            });
        }

        ensure_consumed(bytes, offset)?;
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
    /// V0 messages support address lookup tables
    pub fn decode_v0_message(
        bytes: &[u8],
        signatures: Vec<SignatureBytes>,
    ) -> Result<VersionedTransaction> {
        if bytes.len() < 3 {
            return Err(SolanaError::DeserializationError(
                "V0 message too short".to_string(),
            ));
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
            return Err(SolanaError::DeserializationError(
                "Message too short: no account count".to_string(),
            ));
        }
        let (account_count, len_bytes_consumed) =
            crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
        offset += len_bytes_consumed;

        if offset + (account_count * 32) > bytes.len() {
            return Err(SolanaError::DeserializationError(
                "Message too short: not enough bytes for accounts".to_string(),
            ));
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
            return Err(SolanaError::DeserializationError(
                "Message too short: no recent blockhash".to_string(),
            ));
        }
        let mut recent_blockhash = [0u8; 32];
        recent_blockhash.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        // Instructions
        if offset >= bytes.len() {
            return Err(SolanaError::DeserializationError(
                "Message too short: no instruction count".to_string(),
            ));
        }
        let (instruction_count, len_bytes_consumed) =
            crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
        offset += len_bytes_consumed;

        // Each instruction needs >= 3 bytes; reject counts that can't fit in what's left.
        let remaining = bytes.len().saturating_sub(offset);
        if instruction_count.saturating_mul(3) > remaining {
            return Err(SolanaError::DeserializationError(
                "Message too short: instruction count exceeds remaining bytes".to_string(),
            ));
        }

        let mut instructions = Vec::with_capacity(instruction_count);
        for _ in 0..instruction_count {
            if offset >= bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: incomplete instruction".to_string(),
                ));
            }

            // Program ID index (1 byte)
            let program_id_index = bytes[offset];
            offset += 1;

            if offset >= bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: no account indices count".to_string(),
                ));
            }

            // Account indices (compact-u16 length, then count bytes)
            let (account_indices_count, len_bytes_consumed) =
                crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
            offset += len_bytes_consumed;

            if offset + account_indices_count > bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: not enough account indices".to_string(),
                ));
            }

            let accounts = bytes[offset..offset + account_indices_count].to_vec();
            offset += account_indices_count;

            if offset >= bytes.len() {
                // This check ensures there's at least one byte for the length itself.
                return Err(SolanaError::DeserializationError(
                    "Message too short: no instruction data length".to_string(),
                ));
            }

            // Instruction data (compact-u16 length, then length bytes)
            let (data_length, len_bytes_consumed) =
                crate::decode_compact_u16_len(&bytes[offset..]).map_err(SolanaError::from)?;
            offset += len_bytes_consumed;

            if offset + data_length > bytes.len() {
                return Err(SolanaError::DeserializationError(
                    "Message too short: not enough instruction data".to_string(),
                ));
            }

            let data = bytes[offset..offset + data_length].to_vec();
            offset += data_length;

            instructions.push(CompiledInstruction {
                program_id_index,
                accounts,
                data,
            });
        }

        // The lookup count is required, even when zero.
        let mut address_table_lookups = Vec::new();
        {
            let (lookup_table_count, len_bytes_consumed) =
                crate::decode_compact_u16_len(&bytes[offset..])
                    .map_err(|e| SolanaError::DeserializationError(e.to_string()))?;
            offset += len_bytes_consumed;

            for _ in 0..lookup_table_count {
                if offset + 32 > bytes.len() {
                    return Err(SolanaError::DeserializationError(
                        "Message too short: incomplete address lookup table".to_string(),
                    ));
                }

                // Lookup table account key
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[offset..offset + 32]);
                let lookup_table_key = Pubkey::new(key);
                offset += 32;

                // Writable indexes
                if offset >= bytes.len() {
                    return Err(SolanaError::DeserializationError(
                        "Message too short: no writable indexes count".to_string(),
                    ));
                }
                let (writable_indexes_count, len_bytes_consumed) =
                    crate::decode_compact_u16_len(&bytes[offset..])
                        .map_err(|e| SolanaError::DeserializationError(e.to_string()))?;
                offset += len_bytes_consumed;

                if offset + writable_indexes_count > bytes.len() {
                    return Err(SolanaError::DeserializationError(
                        "Message too short: not enough writable indexes".to_string(),
                    ));
                }

                let writable_indexes = bytes[offset..offset + writable_indexes_count].to_vec();
                offset += writable_indexes_count;

                // Readonly indexes
                if offset >= bytes.len() {
                    return Err(SolanaError::DeserializationError(
                        "Message too short: no readonly indexes count".to_string(),
                    ));
                }
                let (readonly_indexes_count, len_bytes_consumed) =
                    crate::decode_compact_u16_len(&bytes[offset..])
                        .map_err(|e| SolanaError::DeserializationError(e.to_string()))?;
                offset += len_bytes_consumed;

                if offset + readonly_indexes_count > bytes.len() {
                    return Err(SolanaError::DeserializationError(
                        "Message too short: not enough readonly indexes".to_string(),
                    ));
                }

                let readonly_indexes = bytes[offset..offset + readonly_indexes_count].to_vec();
                offset += readonly_indexes_count;

                address_table_lookups.push(MessageAddressTableLookup {
                    account_key: lookup_table_key,
                    writable_indexes,
                    readonly_indexes,
                });
            }
        }

        ensure_consumed(bytes, offset)?;
        Ok(VersionedTransaction::V0 {
            signatures,
            message: VersionedMessageV0 {
                header,
                account_keys,
                recent_blockhash,
                instructions,
                address_table_lookups,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instructions::system,
        types::{Pubkey, SignatureBytes},
    };
    use base64::{Engine, engine::general_purpose::STANDARD};

    /// Legacy tx with SetComputeUnitLimit(420000) and SetComputeUnitPrice(70000).
    const LEGACY_TX: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAgWAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEbrtjJdvWJAv9GZTGL8LaZtMvDe4j2ery4z7rOkRbioxZflXLFqWqlAt1REFSiam0ljvfB1tbBruEpGRTcUQIyQ+ddH9NRneQZQXje5U/3c4cZ2f1JESi76CvBvRoQ6I1LeNzfZ4ZONkowCnqCyeo5+D6Q21gn3U7HVw/KD3HyUW5gVpu5F8ZojWkXLg/+3N6q3ojiaqYyBIbz7VP7jS5Yktrxv5b22C/EFSDs5jUPA7Gz3GLdBNs0iwBHlqUqNEeyNpDX0HWNHV2LiVDOx6m018ea6P+1xroNvWKhmDeTW7oqHXAEK1ih5IO68BBiiKqWNR5VZdBgBsnR+rZKfpfuyE3yQziYO+SoWzCXuvQLyVcRCNKJrACzaN8XXUR1z3rOt8T1lYUIIAQS7tqgcLRsn18N4vVQgXQyv3bQWjh3JtpQT3Bgy9N9myGC4PDjGuVnx2Y7mF4eqlysb0rgrdrB2+FMK6YBPXtlXF4QPTY6rEe+hxkBpCoGK7UJu5BHUK4gJhAewgMolkoyq6sTbFQFuR86447k9ky2veh5uGg40gAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAjJclj04kifG7PRApFI4NgwtaE5na/xCEBI572Nvp+FkDBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAMb6evO+2606PWXzaqvJdDGxu+TC0vbg5HymAgNFL11hBUpTWpkpIQZNJOhxYNo4fHw1td28kruB5B+oQEEFRI0Gm4hX/quBhPtof2NGGMA12sQ53BrrO1WYoPAAAAAAAQbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpDgNoX46QkFPkWBIcZvWnau3HcGqhHIL4qpUqjyt4ealuCa42Moiy1mB8REcWJlkis4eCMyKfY2HMRfldn8r2XwcQAAUCoGgGABAACQNwEQEAAAAAAA8GAAYAEw4UAQAVERQUEgAHExEGCQoCBAULDAgBMSsE7QsayR5iC50OAAAAAAA8XqkAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEBAAAABgIUAwYAAAEJFAMKAwAJA8wSAAAAAAAADgIADQwCAAAAODEAAAAAAAA=";

    /// https://solscan.io/tx/2DZEgrPpdwCu2JcQZJCFivcmLSNMHMmDth9ujqFFZ8UeaEX6EqJmFTfZ43c7LgWqu85wiFhqo2h8PukruvpS4g4u
    const MAYAN_V0_TX: &str = "ATzYOiofQZSWsNe3SxxEPip+Xp9A2Fji+h0xfs7FkmvQxNNgwjeEbTlMr7+e42q9vcvExw2CX4PgNBRuY77O+waAAQAEDPlBHYJN7SVAqQdmNtdFQsCIDVJuEnf59VTtTCOGI7yLh4jpmImexNtJSORTO+sbJ63Aysdx88si41jIW1Wf65qHxwlVbaZ8xI24o/VzmleK1NqPB2lMTcy78ZFbqJ6agIqQAqWC7XmuIVDA/VxhSMZPxFOazPZMJbWyD+TYtXxA3sS/qzC61MydFxPOY3xt62Ug5Tp3r/hC0NimkXNfrMH0UmoX+WTY7c2jVeACjg8EqVgtZZSXgaQRvotGaelPhCySBd5s0S8tvrZZSGGBUknE3Jjh4aGsgXpNY0QHkFnJayU0QDsmAQ7sF/E5yI6Oq1k8w8tnKB6wJR28JzZwp3KVGAf9PgfpG6VoBYOYtT4QWhLzz8wJo5Da/9f9tVVfo7Qj5Z1paZLqq3kUJ1PAm9bYE1qpQE9jUkcSHEnSn0OVAwZGb+UhFzL/7K26csOb57yM5bvF9xJrLEObOkAAAAAGTCSuZOXkbU4/LKndRkF4gm16E7to0DdpTPoefoS0rYF08m4FFLws+yIpIkWYIyALDIz0sekCn1BgZGSqLNo5CwsAF0FkUEJ2ZE5kVGxlWmNsc25JeDVkeUExCgAFAhxCBwAKAAkDBBcBAAAAAAAJBxUABgUWGRQACQcVAAEAGxkUAQEJAxkAAQwCAAAAC/UHPQYAAAAJAhQBAREJBxUAAwAWGRQBAQgoHAABAxsWFBQGHRwhACITAQMPERIUBBACBxweAA4gHw0MAQMbFhQUGjIBLQAAALtk+swxxK8UC/UHPQYAAAD8nvqKAAAAAGQAAAAAAAIAAAAaQAYAAl8A0CAAAgkEFAEAAAEJCQoYAAAFBgMWFxQZxgEgTCkMJ6KE2yRjS4oAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAhOLnPgTBemY6Iqc117gEL72WnXMeAAAAAAAAAAAAAAAAAIM1ifzW7bbgj0x8MtT3G1S9oCkT/ySLiQAAAAAAAAAAAAAAAG4DAAAAAAAA8dcFAAAAAAA7a4ppAAAAAAAAAAAAAAAAAAAAAN3bmpXkQ6IE64ZQ1epXjtcH/iEjAAMC0SsrhG32PclzqA5blk8lqOrlLpR5OoOt60ksQpGLgw4D2cMN+5YRja4DNaX11bThHmwP8vCzdRYtSPXpFGWT2KsACgUGESAhKSowMTQme3jXiWKuyj4qkRn+CZK3WspZpXBM+tnHyaYm4WA/BAMICgsDBgkMIbxt+8RM8X78HZP9nB+0Ah2xfOX9io4UH0AdkLgPT00Fdnd6fH4CdHU=";

    fn decode_legacy_tx() -> VersionedTransaction {
        let data = STANDARD.decode(LEGACY_TX).unwrap();
        VersionedTransaction::deserialize_with_version(&data).unwrap()
    }

    fn decode_mayan_tx() -> VersionedTransaction {
        let data = STANDARD.decode(MAYAN_V0_TX).unwrap();
        VersionedTransaction::deserialize_with_version(&data).unwrap()
    }

    #[test]
    fn decode_legacy() {
        let tx = decode_legacy_tx();
        assert!(matches!(tx, VersionedTransaction::Legacy { .. }));
        assert_eq!(tx.signatures().len(), 1);
        assert_eq!(tx.account_keys().len(), 22);
        assert_eq!(tx.instructions().len(), 7);
    }

    #[test]
    fn decode_v0() {
        let tx = decode_mayan_tx();
        assert!(matches!(tx, VersionedTransaction::V0 { .. }));
        assert_eq!(tx.signatures().len(), 1);
    }

    #[test]
    fn signatures_accessors() {
        let mut tx = decode_legacy_tx();
        let original_sig = tx.signatures()[0];

        tx.signatures_mut()[0] = SignatureBytes::new([42; 64]);
        assert_eq!(tx.signatures()[0], SignatureBytes::new([42; 64]));
        assert_ne!(tx.signatures()[0], original_sig);
    }

    #[test]
    fn get_compute_unit_price_from_legacy() {
        assert_eq!(decode_legacy_tx().get_compute_unit_price(), Some(70_000));
    }

    #[test]
    fn get_compute_unit_price_from_v0() {
        assert_eq!(decode_mayan_tx().get_compute_unit_price(), Some(71_428));
    }

    #[test]
    fn set_compute_unit_price_legacy() {
        let mut tx = decode_legacy_tx();
        assert!(tx.set_compute_unit_price(999_999).unwrap());
        assert_eq!(tx.get_compute_unit_price(), Some(999_999));
    }

    #[test]
    fn get_compute_unit_limit_from_legacy() {
        assert_eq!(decode_legacy_tx().get_compute_unit_limit(), Some(420_000));
    }

    #[test]
    fn get_compute_unit_limit_from_v0() {
        assert_eq!(decode_mayan_tx().get_compute_unit_limit(), Some(475_676));
    }

    #[test]
    fn set_compute_unit_limit_legacy() {
        let mut tx = decode_legacy_tx();
        assert!(tx.set_compute_unit_limit(500_000).unwrap());
        assert_eq!(tx.get_compute_unit_limit(), Some(500_000));
    }

    #[test]
    fn add_instruction_appends_to_legacy() {
        let mut tx = decode_legacy_tx();
        let initial_ix_count = tx.instructions().len();
        let price_before = tx.get_compute_unit_price().unwrap();

        let from = tx.account_keys()[0];
        let to = Pubkey::new([99; 32]);
        tx.add_instruction(system::transfer(&from, &to, 5000))
            .unwrap();

        assert_eq!(tx.instructions().len(), initial_ix_count + 1);
        assert!(tx.account_keys().contains(&to));
        assert_eq!(tx.get_compute_unit_price(), Some(price_before));
    }

    #[test]
    fn add_instruction_errors_on_v0() {
        let mut tx = decode_mayan_tx();
        let from = tx.account_keys()[0];
        let to = Pubkey::new([2; 32]);
        assert!(
            tx.add_instruction(system::transfer(&from, &to, 100))
                .is_err()
        );
    }

    /// A legacy transaction with `num_signatures` zeroed signatures, `header`,
    /// `num_accounts` distinct keys, and a zero blockhash; instructions are appended by callers.
    fn legacy_tx_prefix(num_signatures: u8, header: [u8; 3], num_accounts: u8) -> Vec<u8> {
        let mut bytes = vec![num_signatures];
        bytes.extend(std::iter::repeat_n(0u8, 64 * num_signatures as usize));
        bytes.extend_from_slice(&header);
        bytes.push(num_accounts);
        for i in 0..num_accounts {
            bytes.extend_from_slice(&[i + 1; 32]);
        }
        bytes.extend_from_slice(&[0u8; 32]);
        bytes
    }

    fn assert_rejected(bytes: &[u8], reason: &str) {
        assert!(
            VersionedTransaction::deserialize_with_version(bytes).is_err(),
            "{reason}"
        );
    }

    #[test]
    fn deserialize_accepts_minimal_transactions() {
        // 1 signer, program at index 1, one instruction touching the payer.
        let mut legacy = legacy_tx_prefix(1, [1, 0, 1], 2);
        legacy.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert!(VersionedTransaction::deserialize_with_version(&legacy).is_ok());

        let mut v0 = legacy_tx_prefix(1, [1, 0, 1], 2);
        v0.insert(65, 0x80);
        v0.extend_from_slice(&[1, 1, 1, 0, 0]);
        v0.push(0); // address table lookup count
        assert!(VersionedTransaction::deserialize_with_version(&v0).is_ok());
    }

    #[test]
    fn deserialize_rejects_truncation_and_trailing_bytes() {
        for fixture in [LEGACY_TX, MAYAN_V0_TX] {
            let data = STANDARD.decode(fixture).unwrap();
            for len in 0..data.len() {
                assert_rejected(&data[..len], &format!("truncated to {len} bytes"));
            }
            let mut padded = data.clone();
            padded.push(0);
            assert_rejected(&padded, "trailing byte");
        }
    }

    #[test]
    fn deserialize_requires_v0_lookup_count() {
        let data = STANDARD.decode(MAYAN_V0_TX).unwrap();
        let mut tx = VersionedTransaction::deserialize_with_version(&data).unwrap();
        if let VersionedTransaction::V0 { message, .. } = &mut tx {
            message.address_table_lookups.clear();
        }
        let mut bytes = tx.serialize().unwrap();
        assert_eq!(bytes.pop(), Some(0));
        assert_rejected(&bytes, "missing address table lookup count");
    }

    #[test]
    fn deserialize_rejects_unknown_discriminators() {
        let mut v0 = legacy_tx_prefix(1, [1, 0, 1], 2);
        v0.extend_from_slice(&[1, 1, 1, 0, 0, 0]);
        // A v0 message without the signature envelope, and the off-chain message prefix.
        for first in [0x80, 0x81, 0xff] {
            let mut bytes = v0[65..].to_vec();
            bytes.insert(0, first);
            assert_rejected(&bytes, &format!("discriminator {first:#04x}"));
        }
    }

    #[test]
    fn deserialize_rejects_non_canonical_lengths() {
        // Account count 2 encoded as the alias [0x82, 0x00].
        let mut bytes = legacy_tx_prefix(1, [1, 0, 1], 2);
        bytes.splice(68..69, [0x82, 0x00]);
        bytes.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_rejected(&bytes, "non-canonical account count");
    }

    #[test]
    fn deserialize_rejects_unsanitary_messages() {
        let cases: [([u8; 3], u8, &[u8], &str); 8] = [
            (
                [1, 0, 5],
                2,
                &[1, 1, 1, 0, 0],
                "readonly unsigned exceeds keys",
            ),
            (
                [1, 0, 2],
                2,
                &[1, 1, 1, 0, 0],
                "signers + readonly unsigned exceed keys",
            ),
            ([1, 1, 1], 2, &[1, 1, 1, 0, 0], "no writable fee payer"),
            (
                [1, 2, 0],
                2,
                &[1, 1, 1, 0, 0],
                "readonly signed exceeds signers",
            ),
            ([0, 0, 1], 2, &[1, 1, 1, 0, 0], "no signers"),
            ([1, 0, 1], 2, &[1, 0, 1, 0, 0], "fee payer as program"),
            ([1, 0, 1], 2, &[1, 2, 1, 0, 0], "program index out of range"),
            ([1, 0, 1], 2, &[1, 1, 1, 2, 0], "account index out of range"),
        ];
        for (header, num_accounts, instruction, reason) in cases {
            let num_signatures = header[0];
            let mut bytes = legacy_tx_prefix(num_signatures, header, num_accounts);
            bytes.extend_from_slice(instruction);
            assert_rejected(&bytes, reason);
        }

        let mut too_many_signatures = legacy_tx_prefix(2, [1, 0, 1], 2);
        too_many_signatures.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_rejected(&too_many_signatures, "signature count mismatch");

        let mut huge_instruction_count = legacy_tx_prefix(1, [1, 0, 1], 2);
        huge_instruction_count
            .extend_from_slice(&crate::encode_length_to_compact_u16_bytes(60_000).unwrap());
        assert_rejected(&huge_instruction_count, "instruction count exceeds input");
    }

    #[test]
    fn deserialize_rejects_unsanitary_v0_lookups() {
        let v0_tx = |instruction: &[u8], lookups: &[u8]| {
            let mut bytes = legacy_tx_prefix(1, [1, 0, 1], 2);
            bytes.insert(65, 0x80);
            bytes.extend_from_slice(instruction);
            bytes.extend_from_slice(lookups);
            bytes
        };
        let mut empty_lookup = vec![1];
        empty_lookup.extend_from_slice(&[9; 32]);
        empty_lookup.extend_from_slice(&[0, 0]);
        assert_rejected(
            &v0_tx(&[1, 1, 1, 0, 0], &empty_lookup),
            "lookup without indexes",
        );

        let mut one_lookup = vec![1];
        one_lookup.extend_from_slice(&[9; 32]);
        one_lookup.extend_from_slice(&[1, 0, 0]);
        // Index 2 is the looked-up account; programs may not be loaded from tables.
        assert!(
            VersionedTransaction::deserialize_with_version(&v0_tx(&[1, 1, 1, 2, 0], &one_lookup))
                .is_ok()
        );
        assert_rejected(
            &v0_tx(&[1, 2, 1, 0, 0], &one_lookup),
            "program loaded from lookup table",
        );
        assert_rejected(
            &v0_tx(&[1, 1, 1, 3, 0], &one_lookup),
            "account index beyond loaded keys",
        );
    }

    #[test]
    fn serialize_roundtrip_legacy() {
        let data = STANDARD.decode(LEGACY_TX).unwrap();
        let tx = VersionedTransaction::deserialize_with_version(&data).unwrap();

        let reserialized = tx.serialize().unwrap();
        assert_eq!(reserialized, data, "byte-exact roundtrip failed");

        let tx2 = VersionedTransaction::deserialize_with_version(&reserialized).unwrap();
        assert!(matches!(tx2, VersionedTransaction::Legacy { .. }));
        assert_eq!(tx2.get_compute_unit_price(), Some(70_000));
        assert_eq!(tx2.get_compute_unit_limit(), Some(420_000));
    }

    #[test]
    fn serialize_roundtrip_v0() {
        let data = STANDARD.decode(MAYAN_V0_TX).unwrap();
        let tx = VersionedTransaction::deserialize_with_version(&data).unwrap();

        let reserialized = tx.serialize().unwrap();
        assert_eq!(reserialized, data, "byte-exact roundtrip failed");

        let tx2 = VersionedTransaction::deserialize_with_version(&reserialized).unwrap();
        assert!(matches!(tx2, VersionedTransaction::V0 { .. }));
        assert_eq!(tx2.get_compute_unit_price(), Some(71_428));
        assert_eq!(tx2.get_compute_unit_limit(), Some(475_676));
    }

    #[test]
    fn sign_and_roundtrip() {
        let mut tx = decode_legacy_tx();
        let private_key = [1u8; 32];

        let message_bytes = tx.serialize_message().unwrap();
        let sig = sign_message(&private_key, &message_bytes).unwrap();
        tx.signatures_mut()[0] = sig;

        let bytes = tx.serialize().unwrap();
        let deserialized = VersionedTransaction::deserialize_with_version(&bytes).unwrap();
        assert_eq!(deserialized.signatures()[0], sig);
        assert_ne!(deserialized.signatures()[0], SignatureBytes::default());
    }
}
