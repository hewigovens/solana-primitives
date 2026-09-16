use crate::compiler::{CompiledKeys, compile_instructions};
use crate::error::{Result, SanitizeError};
use crate::types::{
    AddressLookupTableAccount, CompiledInstruction, Instruction, MessageAddressTableLookup, Pubkey,
};
use crate::wire;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Account indexes are `u8`, so a message can reference at most 256 accounts.
const MAX_ACCOUNT_KEYS: usize = 256;

/// The message header, identifying signed and read-only `account_keys`.
///
/// Account keys are ordered writable signers (fee payer first), readonly
/// signers, writable non-signers, then readonly non-signers.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
pub struct MessageHeader {
    /// The number of signatures required for this message to be considered valid.
    pub num_required_signatures: u8,
    /// The last `num_readonly_signed_accounts` of the signed keys are read-only accounts.
    pub num_readonly_signed_accounts: u8,
    /// The last `num_readonly_unsigned_accounts` of the unsigned keys are read-only accounts.
    pub num_readonly_unsigned_accounts: u8,
}

impl MessageHeader {
    /// Rules shared by every message version.
    fn sanitize(&self, num_account_keys: usize) -> std::result::Result<(), SanitizeError> {
        if usize::from(self.num_required_signatures)
            + usize::from(self.num_readonly_unsigned_accounts)
            > num_account_keys
        {
            return Err(SanitizeError::NotEnoughAccountKeys);
        }
        if self.num_readonly_signed_accounts >= self.num_required_signatures {
            return Err(SanitizeError::NoWritableFeePayer);
        }
        Ok(())
    }
}

/// Check instruction indexes: programs must be static non-payer keys, and
/// accounts must be below `num_account_keys`.
fn sanitize_instructions(
    instructions: &[CompiledInstruction],
    num_static_keys: usize,
    num_account_keys: usize,
) -> std::result::Result<(), SanitizeError> {
    for instruction in instructions {
        let program_index = usize::from(instruction.program_id_index);
        if program_index == 0 || program_index >= num_static_keys {
            return Err(SanitizeError::InvalidProgramIndex);
        }
        if instruction
            .accounts
            .iter()
            .any(|&index| usize::from(index) >= num_account_keys)
        {
            return Err(SanitizeError::InvalidAccountIndex);
        }
    }
    Ok(())
}

/// A legacy transaction message.
///
/// Its wire encoding is `header || compact-u16 keys || blockhash || compact-u16 instructions`,
/// which is also the byte string that signers sign.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
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

/// The legacy message; kept as an alias of [`Message`].
pub type LegacyMessage = Message;

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

    /// Compile `instructions` with `payer` as the fee payer.
    ///
    /// Keys are ordered like the Solana SDK's `Message::new`.
    pub fn try_compile(
        payer: &Pubkey,
        instructions: &[Instruction],
        recent_blockhash: [u8; 32],
    ) -> Result<Self> {
        let (header, account_keys) =
            CompiledKeys::compile(payer, instructions).into_message_components()?;
        let instructions = compile_instructions(instructions, &account_keys, &[])?;
        Ok(Self {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        })
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

    /// The fee payer (first account key).
    pub fn fee_payer(&self) -> Option<&Pubkey> {
        self.account_keys.first()
    }

    /// Serialize to wire bytes. These are the bytes that get signed.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        wire::write_legacy_message(&mut out, self)?;
        Ok(out)
    }

    /// Serialize to the bytes that get signed; same as [`Message::serialize`].
    pub fn serialize_for_signing(&self) -> Result<Vec<u8>> {
        self.serialize()
    }

    /// Check Solana's legacy message sanitization rules.
    pub fn sanitize(&self) -> Result<()> {
        let num_keys = self.account_keys.len();
        self.header.sanitize(num_keys)?;
        sanitize_instructions(&self.instructions, num_keys, num_keys)?;
        Ok(())
    }
}

/// A v0 message, which can load accounts from address lookup tables.
///
/// Its wire encoding is `0x80 || legacy body || compact-u16 lookups`.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub struct MessageV0 {
    /// The message header, identifying signed and read-only `account_keys`.
    pub header: MessageHeader,
    /// Static account keys; looked-up keys follow them in index space.
    pub account_keys: Vec<Pubkey>,
    /// The blockhash of a recent block.
    pub recent_blockhash: [u8; 32],
    /// Instructions that will be executed in sequence and committed in one atomic transaction if all succeed.
    pub instructions: Vec<CompiledInstruction>,
    /// List of address lookup table references
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

/// The v0 message; kept as an alias of [`MessageV0`].
pub type VersionedMessageV0 = MessageV0;

impl MessageV0 {
    /// Compile `instructions` with `payer` as the fee payer, loading eligible
    /// accounts from `address_lookup_tables`.
    ///
    /// Tables are tried in order and each account is loaded from the first table
    /// (and first index) that has it. Signers, invoked programs, and the durable
    /// nonce account always stay static. Matches the Solana SDK's `v0::Message::try_compile`.
    pub fn try_compile(
        payer: &Pubkey,
        instructions: &[Instruction],
        address_lookup_tables: &[AddressLookupTableAccount],
        recent_blockhash: [u8; 32],
    ) -> Result<Self> {
        let mut keys = CompiledKeys::compile(payer, instructions);
        let mut address_table_lookups = Vec::new();
        let mut loaded = Vec::new();
        for table in address_lookup_tables {
            if let Some((lookup, addresses)) = keys.extract_table_lookup(table)? {
                address_table_lookups.push(lookup);
                loaded.push(addresses);
            }
        }
        let (header, account_keys) = keys.into_message_components()?;
        let instructions = compile_instructions(instructions, &account_keys, &loaded)?;
        Ok(Self {
            header,
            account_keys,
            recent_blockhash,
            instructions,
            address_table_lookups,
        })
    }

    /// Serialize to wire bytes, including the `0x80` version prefix. These are the bytes that get signed.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        wire::write_v0_message(&mut out, self)?;
        Ok(out)
    }

    /// Serialize to the bytes that get signed; same as [`MessageV0::serialize`].
    pub fn serialize_for_signing(&self) -> Result<Vec<u8>> {
        self.serialize()
    }

    /// Check Solana's v0 message sanitization rules.
    pub fn sanitize(&self) -> Result<()> {
        let num_static_keys = self.account_keys.len();
        self.header.sanitize(num_static_keys)?;

        let mut num_lookup_keys = 0;
        for lookup in &self.address_table_lookups {
            let num_indexes = lookup.writable_indexes.len() + lookup.readonly_indexes.len();
            if num_indexes == 0 {
                return Err(SanitizeError::EmptyAddressTableLookup.into());
            }
            num_lookup_keys += num_indexes;
        }
        let num_keys = num_static_keys + num_lookup_keys;
        if num_keys > MAX_ACCOUNT_KEYS {
            return Err(SanitizeError::TooManyAccountKeys.into());
        }

        // Programs cannot be loaded from lookup tables.
        sanitize_instructions(&self.instructions, num_static_keys, num_keys)?;
        Ok(())
    }
}

/// A message of any supported version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionedMessage {
    /// Legacy message (no version prefix).
    Legacy(Message),
    /// Version 0 message.
    V0(MessageV0),
}

impl Default for VersionedMessage {
    fn default() -> Self {
        Self::Legacy(Message::default())
    }
}

impl From<Message> for VersionedMessage {
    fn from(message: Message) -> Self {
        Self::Legacy(message)
    }
}

impl From<MessageV0> for VersionedMessage {
    fn from(message: MessageV0) -> Self {
        Self::V0(message)
    }
}

impl VersionedMessage {
    /// The message header.
    pub fn header(&self) -> &MessageHeader {
        match self {
            Self::Legacy(message) => &message.header,
            Self::V0(message) => &message.header,
        }
    }

    /// Account keys stored in the message itself (excludes looked-up keys).
    pub fn static_account_keys(&self) -> &[Pubkey] {
        match self {
            Self::Legacy(message) => &message.account_keys,
            Self::V0(message) => &message.account_keys,
        }
    }

    /// The recent blockhash (or durable nonce) that bounds the message lifetime.
    pub fn recent_blockhash(&self) -> &[u8; 32] {
        match self {
            Self::Legacy(message) => &message.recent_blockhash,
            Self::V0(message) => &message.recent_blockhash,
        }
    }

    /// Replace the recent blockhash. Existing signatures become invalid.
    pub fn set_recent_blockhash(&mut self, recent_blockhash: [u8; 32]) {
        match self {
            Self::Legacy(message) => message.recent_blockhash = recent_blockhash,
            Self::V0(message) => message.recent_blockhash = recent_blockhash,
        }
    }

    /// Compiled instructions.
    pub fn instructions(&self) -> &[CompiledInstruction] {
        match self {
            Self::Legacy(message) => &message.instructions,
            Self::V0(message) => &message.instructions,
        }
    }

    /// Address table lookups; `None` for versions without lookup tables.
    pub fn address_table_lookups(&self) -> Option<&[MessageAddressTableLookup]> {
        match self {
            Self::Legacy(_) => None,
            Self::V0(message) => Some(&message.address_table_lookups),
        }
    }

    /// The fee payer (first account key).
    pub fn fee_payer(&self) -> Option<&Pubkey> {
        self.static_account_keys().first()
    }

    /// Whether the account at `index` must sign.
    pub fn is_signer(&self, index: usize) -> bool {
        index < usize::from(self.header().num_required_signatures)
    }

    /// Serialize to wire bytes. These are the bytes that get signed.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        wire::write_message(&mut out, self)?;
        Ok(out)
    }

    /// Decode exactly one message and check sanitization rules.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let message = wire::decode_message(bytes)?;
        message.sanitize()?;
        Ok(message)
    }

    /// Check Solana's sanitization rules for this message version.
    pub fn sanitize(&self) -> Result<()> {
        match self {
            Self::Legacy(message) => message.sanitize(),
            Self::V0(message) => message.sanitize(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DecodeError, SolanaError};

    fn header(
        num_required_signatures: u8,
        readonly_signed: u8,
        readonly_unsigned: u8,
    ) -> MessageHeader {
        MessageHeader {
            num_required_signatures,
            num_readonly_signed_accounts: readonly_signed,
            num_readonly_unsigned_accounts: readonly_unsigned,
        }
    }

    fn instruction(program_id_index: u8, accounts: &[u8]) -> CompiledInstruction {
        CompiledInstruction {
            program_id_index,
            accounts: accounts.to_vec(),
            data: vec![],
        }
    }

    fn legacy(
        header: MessageHeader,
        num_keys: u8,
        instructions: Vec<CompiledInstruction>,
    ) -> Message {
        Message::new(
            header,
            (0..num_keys).map(|i| Pubkey::new([i; 32])).collect(),
            [0; 32],
            instructions,
        )
    }

    fn sanitize_err(message: impl Into<VersionedMessage>) -> SanitizeError {
        match message.into().sanitize() {
            Err(SolanaError::Sanitize(err)) => err,
            other => panic!("expected a sanitize error, got {other:?}"),
        }
    }

    #[test]
    fn legacy_sanitize_rules() {
        use SanitizeError::*;
        assert_eq!(
            legacy(header(1, 0, 1), 2, vec![instruction(1, &[0])]).sanitize(),
            Ok(())
        );
        let cases = [
            (
                header(1, 0, 5),
                2,
                instruction(1, &[0]),
                NotEnoughAccountKeys,
            ),
            (
                header(1, 0, 2),
                2,
                instruction(1, &[0]),
                NotEnoughAccountKeys,
            ),
            (header(1, 1, 1), 2, instruction(1, &[0]), NoWritableFeePayer),
            (header(0, 0, 1), 2, instruction(1, &[0]), NoWritableFeePayer),
            (
                header(1, 0, 1),
                2,
                instruction(0, &[0]),
                InvalidProgramIndex,
            ),
            (
                header(1, 0, 1),
                2,
                instruction(2, &[0]),
                InvalidProgramIndex,
            ),
            (
                header(1, 0, 1),
                2,
                instruction(1, &[2]),
                InvalidAccountIndex,
            ),
        ];
        for (header, num_keys, instruction, expected) in cases {
            assert_eq!(
                sanitize_err(legacy(header, num_keys, vec![instruction])),
                expected
            );
        }
    }

    #[test]
    fn v0_sanitize_rules() {
        let lookup = |writable: &[u8], readonly: &[u8]| MessageAddressTableLookup {
            account_key: Pubkey::new([9; 32]),
            writable_indexes: writable.to_vec(),
            readonly_indexes: readonly.to_vec(),
        };
        let v0 = |instructions: Vec<CompiledInstruction>, lookups| MessageV0 {
            header: header(1, 0, 1),
            account_keys: vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])],
            recent_blockhash: [0; 32],
            instructions,
            address_table_lookups: lookups,
        };

        // Index 2 is the looked-up key.
        assert_eq!(
            v0(vec![instruction(1, &[0, 2])], vec![lookup(&[0], &[])]).sanitize(),
            Ok(())
        );
        assert_eq!(
            sanitize_err(v0(vec![instruction(2, &[0])], vec![lookup(&[0], &[])])),
            SanitizeError::InvalidProgramIndex
        );
        assert_eq!(
            sanitize_err(v0(vec![instruction(1, &[3])], vec![lookup(&[0], &[])])),
            SanitizeError::InvalidAccountIndex
        );
        assert_eq!(
            sanitize_err(v0(vec![], vec![lookup(&[], &[])])),
            SanitizeError::EmptyAddressTableLookup
        );
        let indexes: Vec<u8> = (0..=254).collect();
        assert_eq!(
            sanitize_err(v0(vec![], vec![lookup(&indexes, &[])])),
            SanitizeError::TooManyAccountKeys
        );
        assert_eq!(
            v0(vec![], vec![lookup(&indexes[1..], &[])]).sanitize(),
            Ok(())
        );
    }

    #[test]
    fn versioned_message_roundtrip() {
        let message =
            VersionedMessage::Legacy(legacy(header(1, 0, 1), 2, vec![instruction(1, &[0])]));
        let bytes = message.serialize().unwrap();
        assert_eq!(VersionedMessage::deserialize(&bytes), Ok(message));

        let v0 = VersionedMessage::V0(MessageV0 {
            header: header(1, 0, 1),
            account_keys: vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])],
            instructions: vec![instruction(1, &[0])],
            ..MessageV0::default()
        });
        let bytes = v0.serialize().unwrap();
        assert_eq!(bytes[0], 0x80);
        assert_eq!(VersionedMessage::deserialize(&bytes), Ok(v0));

        let mut unsupported = bytes.clone();
        unsupported[0] = 0x82;
        assert_eq!(
            VersionedMessage::deserialize(&unsupported),
            Err(DecodeError::UnsupportedMessageVersion(2).into())
        );
    }
}
