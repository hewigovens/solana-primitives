use crate::crypto::{get_public_key, sign_message, verify_signature};
use crate::error::{DecodeError, Result, SanitizeError, SolanaError};
use crate::instructions::compute_budget::ComputeBudgetInstruction;
use crate::instructions::program_ids::{compute_budget_program, system_program};
use crate::instructions::system::is_advance_nonce_instruction_data;
use crate::types::{
    CompiledInstruction, MAX_TRANSACTION_SIZE, Message, MessageAddressTableLookup, MessageHeader,
    Pubkey, SignatureBytes, VersionedMessage, v1,
};
use crate::wire;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Transaction wire format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionVersion {
    /// Unversioned transaction.
    Legacy,
    /// Version 0 (address lookup tables).
    V0,
    /// Version 1 (SIMD-0385: larger transactions, inline compute budget config).
    V1,
}

/// A legacy transaction.
///
/// Serialization, signing, and verification share their implementation with
/// [`VersionedTransaction`].
#[derive(
    Debug, Clone, Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub struct Transaction {
    /// One signature per required signer, in account key order.
    pub signatures: Vec<SignatureBytes>,
    /// The message
    pub message: Message,
}

impl Transaction {
    /// Create an unsigned transaction with placeholder signatures.
    pub fn new(message: Message) -> Self {
        Self {
            signatures: placeholder_signatures(&message.header),
            message,
        }
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

    /// The bytes signers sign.
    pub fn message_data(&self) -> Result<Vec<u8>> {
        self.message.serialize()
    }

    /// Serialize to wire bytes.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(wire::encode_legacy_envelope(&self.signatures, |out| {
            wire::write_legacy_message(out, &self.message)
        })?)
    }

    /// Serialize to wire bytes; same as [`Transaction::serialize`].
    #[deprecated(since = "0.3.0", note = "use `serialize`")]
    pub fn serialize_legacy(&self) -> Result<Vec<u8>> {
        self.serialize()
    }

    /// Decode and sanitize a legacy transaction. Other versions are rejected.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        VersionedTransaction::deserialize(bytes)?
            .into_legacy_transaction()
            .ok_or(DecodeError::UnexpectedVersion.into())
    }

    /// Decode a legacy transaction; same as [`Transaction::deserialize`].
    #[deprecated(since = "0.3.0", note = "use `deserialize`")]
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self> {
        Self::deserialize(bytes)
    }

    /// Sign with every required signer. See [`VersionedTransaction::sign`].
    pub fn sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.sign_with(private_keys, true)
    }

    /// Sign with some of the required signers. See [`VersionedTransaction::partial_sign`].
    pub fn partial_sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.sign_with(private_keys, false)
    }

    fn sign_with(&mut self, private_keys: &[&[u8]], require_all: bool) -> Result<()> {
        self.message.sanitize()?;
        let message_data = self.message_data()?;
        let signers = required_signers(&self.message.header, &self.message.account_keys);
        sign(
            &mut self.signatures,
            signers,
            &message_data,
            private_keys,
            require_all,
        )
    }

    /// Whether every required signature is present (not verified).
    pub fn is_signed(&self) -> bool {
        is_signed(&self.message.header, &self.signatures)
    }

    /// Sanitize and verify every required signature.
    pub fn verify(&self) -> Result<()> {
        self.sanitize()?;
        verify(
            &self.signatures,
            required_signers(&self.message.header, &self.message.account_keys),
            &self.message_data()?,
        )
    }

    /// Check message sanitization rules and the signature count.
    pub fn sanitize(&self) -> Result<()> {
        self.message.sanitize()?;
        sanitize_signature_count(&self.message.header, self.signatures.len())
    }

    /// Check that the wire size is within [`MAX_TRANSACTION_SIZE`].
    pub fn validate_size(&self) -> Result<()> {
        check_size(self.serialize()?.len(), MAX_TRANSACTION_SIZE)
    }
}

/// A transaction of any supported version.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedTransaction {
    /// One signature per required signer, in account key order.
    pub signatures: Vec<SignatureBytes>,
    /// The message.
    pub message: VersionedMessage,
}

impl From<Transaction> for VersionedTransaction {
    fn from(transaction: Transaction) -> Self {
        Self {
            signatures: transaction.signatures,
            message: VersionedMessage::Legacy(transaction.message),
        }
    }
}

impl VersionedTransaction {
    /// Create an unsigned transaction with placeholder signatures.
    pub fn new(message: VersionedMessage) -> Self {
        Self {
            signatures: placeholder_signatures(message.header()),
            message,
        }
    }

    /// The transaction's wire version.
    pub fn version(&self) -> TransactionVersion {
        match self.message {
            VersionedMessage::Legacy(_) => TransactionVersion::Legacy,
            VersionedMessage::V0(_) => TransactionVersion::V0,
            VersionedMessage::V1(_) => TransactionVersion::V1,
        }
    }

    /// The legacy transaction, if this is one.
    pub fn into_legacy_transaction(self) -> Option<Transaction> {
        match self.message {
            VersionedMessage::Legacy(message) => Some(Transaction {
                signatures: self.signatures,
                message,
            }),
            _ => None,
        }
    }

    /// The signatures.
    pub fn signatures(&self) -> &[SignatureBytes] {
        &self.signatures
    }

    /// The message header.
    pub fn header(&self) -> &MessageHeader {
        self.message.header()
    }

    /// Get the number of required signatures
    pub fn num_required_signatures(&self) -> u8 {
        self.header().num_required_signatures
    }

    /// Get the number of read-only signed accounts
    pub fn num_readonly_signed_accounts(&self) -> u8 {
        self.header().num_readonly_signed_accounts
    }

    /// Get the number of read-only unsigned accounts
    pub fn num_readonly_unsigned_accounts(&self) -> u8 {
        self.header().num_readonly_unsigned_accounts
    }

    /// Static account keys (looked-up keys are not included).
    pub fn account_keys(&self) -> &[Pubkey] {
        self.message.static_account_keys()
    }

    /// Get the recent blockhash
    pub fn recent_blockhash(&self) -> &[u8; 32] {
        self.message.recent_blockhash()
    }

    /// Get the instructions
    pub fn instructions(&self) -> &[CompiledInstruction] {
        self.message.instructions()
    }

    /// Address table lookups; `None` for versions without lookup tables.
    pub fn address_table_lookups(&self) -> Option<&[MessageAddressTableLookup]> {
        self.message.address_table_lookups()
    }

    /// Whether the first instruction is a System `AdvanceNonceAccount`.
    pub fn uses_durable_nonce(&self) -> bool {
        self.instructions().first().is_some_and(|instruction| {
            self.program_id(instruction) == Some(system_program())
                && is_advance_nonce_instruction_data(&instruction.data)
        })
    }

    /// The bytes signers sign.
    pub fn serialize_message(&self) -> Result<Vec<u8>> {
        self.message.serialize()
    }

    /// Serialize to wire bytes.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(wire::encode_transaction(self)?)
    }

    /// Decode exactly one transaction and check it the way the network would.
    ///
    /// Truncated input, trailing bytes, non-canonical lengths, unknown versions
    /// or config bits, transactions over their version's size limit, and
    /// transactions that fail [`VersionedTransaction::sanitize`] are rejected.
    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        let transaction = wire::decode_transaction(bytes)?;
        check_size(bytes.len(), transaction.max_size())?;
        transaction.sanitize()?;
        Ok(transaction)
    }

    /// Decode a transaction; same as [`VersionedTransaction::deserialize`].
    #[deprecated(since = "0.3.0", note = "use `deserialize`")]
    pub fn deserialize_with_version(bytes: &[u8]) -> Result<Self> {
        Self::deserialize(bytes)
    }

    /// Check message sanitization rules and the signature count.
    pub fn sanitize(&self) -> Result<()> {
        self.message.sanitize()?;
        sanitize_signature_count(self.header(), self.signatures.len())
    }

    /// The maximum wire size for this transaction's version: 1232 bytes for
    /// legacy and v0, 4096 for v1.
    pub fn max_size(&self) -> usize {
        match self.version() {
            TransactionVersion::Legacy | TransactionVersion::V0 => MAX_TRANSACTION_SIZE,
            TransactionVersion::V1 => v1::MAX_TRANSACTION_SIZE,
        }
    }

    /// Check that the wire size is within [`VersionedTransaction::max_size`].
    pub fn validate_size(&self) -> Result<()> {
        check_size(self.serialize()?.len(), self.max_size())
    }

    /// Sign with every required signer.
    ///
    /// Each key must belong to a required signer; its signature is placed at
    /// that signer's index. Fails if any required signature is still missing.
    pub fn sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.sign_with(private_keys, true)
    }

    /// Sign with some of the required signers, leaving other signatures as they are.
    ///
    /// Each key must belong to a required signer.
    pub fn partial_sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.sign_with(private_keys, false)
    }

    fn sign_with(&mut self, private_keys: &[&[u8]], require_all: bool) -> Result<()> {
        self.message.sanitize()?;
        let message_data = self.serialize_message()?;
        let signers = required_signers(self.message.header(), self.message.static_account_keys());
        sign(
            &mut self.signatures,
            signers,
            &message_data,
            private_keys,
            require_all,
        )
    }

    /// Whether every required signature is present (not verified).
    pub fn is_signed(&self) -> bool {
        is_signed(self.header(), &self.signatures)
    }

    /// Sanitize and verify every required signature.
    pub fn verify(&self) -> Result<()> {
        self.sanitize()?;
        verify(
            &self.signatures,
            required_signers(self.header(), self.account_keys()),
            &self.serialize_message()?,
        )
    }

    fn program_id(&self, instruction: &CompiledInstruction) -> Option<Pubkey> {
        self.account_keys()
            .get(usize::from(instruction.program_id_index))
            .copied()
    }

    /// Compute Budget instructions with their positions in the instruction list.
    /// Always empty for v1, whose runtime ignores them.
    fn compute_budget_instructions(
        &self,
    ) -> impl Iterator<Item = (usize, ComputeBudgetInstruction)> + '_ {
        let program_id = compute_budget_program();
        let instructions = match self.message {
            VersionedMessage::V1(_) => &[],
            _ => self.instructions(),
        };
        instructions
            .iter()
            .enumerate()
            .filter(move |(_, instruction)| self.program_id(instruction) == Some(program_id))
            .filter_map(|(index, instruction)| {
                ComputeBudgetInstruction::parse(&instruction.data).map(|parsed| (index, parsed))
            })
    }

    fn find_compute_budget<T>(
        &self,
        select: impl Fn(ComputeBudgetInstruction) -> Option<T>,
    ) -> Option<T> {
        self.compute_budget_instructions()
            .find_map(|(_, instruction)| select(instruction))
    }

    /// Overwrite the value bytes (after the discriminant) of the first instruction
    /// accepted by `select`, keeping the instruction length unchanged.
    fn replace_compute_budget_value(
        &mut self,
        select: impl Fn(ComputeBudgetInstruction) -> bool,
        value: &[u8],
    ) -> bool {
        let Some((index, _)) = self
            .compute_budget_instructions()
            .find(|(_, instruction)| select(*instruction))
        else {
            return false;
        };
        self.message.instructions_mut()[index].data[1..=value.len()].copy_from_slice(value);
        true
    }

    fn v1_config_mut(&mut self) -> Option<&mut v1::TransactionConfig> {
        match &mut self.message {
            VersionedMessage::V1(message) => Some(&mut message.config),
            _ => None,
        }
    }

    /// The legacy/v0 compute unit price in micro-lamports per compute unit, from the
    /// first `SetComputeUnitPrice` instruction.
    ///
    /// Always `None` for v1, which pays a total fee instead; see
    /// [`VersionedTransaction::priority_fee_lamports`].
    pub fn get_compute_unit_price(&self) -> Option<u64> {
        self.find_compute_budget(|instruction| match instruction {
            ComputeBudgetInstruction::SetComputeUnitPrice(micro_lamports) => Some(micro_lamports),
            _ => None,
        })
    }

    /// Overwrite the first `SetComputeUnitPrice` value; returns `false` if there is none.
    /// Existing signatures become invalid.
    ///
    /// Fails with [`SolanaError::UnsupportedVersion`] for v1; use
    /// [`VersionedTransaction::set_priority_fee_lamports`].
    pub fn set_compute_unit_price(&mut self, micro_lamports: u64) -> Result<bool> {
        if self.version() == TransactionVersion::V1 {
            return Err(SolanaError::UnsupportedVersion);
        }
        Ok(self.replace_compute_budget_value(
            |instruction| {
                matches!(
                    instruction,
                    ComputeBudgetInstruction::SetComputeUnitPrice(_)
                )
            },
            &micro_lamports.to_le_bytes(),
        ))
    }

    /// The v1 total priority fee in lamports; `None` if unset or not v1.
    pub fn priority_fee_lamports(&self) -> Option<u64> {
        self.message.transaction_config()?.priority_fee
    }

    /// Set the v1 total priority fee in lamports. Existing signatures become invalid.
    ///
    /// Fails with [`SolanaError::UnsupportedVersion`] for legacy and v0; use
    /// [`VersionedTransaction::set_compute_unit_price`].
    pub fn set_priority_fee_lamports(&mut self, lamports: u64) -> Result<()> {
        let config = self
            .v1_config_mut()
            .ok_or(SolanaError::UnsupportedVersion)?;
        config.priority_fee = Some(lamports);
        Ok(())
    }

    /// The requested compute unit limit: the v1 config value, or the first
    /// `SetComputeUnitLimit` instruction for legacy and v0.
    pub fn get_compute_unit_limit(&self) -> Option<u32> {
        if let Some(config) = self.message.transaction_config() {
            return config.compute_unit_limit;
        }
        self.find_compute_budget(|instruction| match instruction {
            ComputeBudgetInstruction::SetComputeUnitLimit(units) => Some(units),
            _ => None,
        })
    }

    /// Set the compute unit limit; returns `false` if a legacy/v0 transaction has no
    /// `SetComputeUnitLimit` instruction to overwrite. v1 always sets its config.
    /// Existing signatures become invalid.
    pub fn set_compute_unit_limit(&mut self, units: u32) -> Result<bool> {
        if let Some(config) = self.v1_config_mut() {
            config.compute_unit_limit = Some(units);
            return Ok(true);
        }
        Ok(self.replace_compute_budget_value(
            |instruction| {
                matches!(
                    instruction,
                    ComputeBudgetInstruction::SetComputeUnitLimit(_)
                )
            },
            &units.to_le_bytes(),
        ))
    }

    /// The requested loaded accounts data size limit: the v1 config value, or the
    /// first `SetLoadedAccountsDataSizeLimit` instruction for legacy and v0.
    pub fn get_loaded_accounts_data_size_limit(&self) -> Option<u32> {
        if let Some(config) = self.message.transaction_config() {
            return config.loaded_accounts_data_size_limit;
        }
        self.find_compute_budget(|instruction| match instruction {
            ComputeBudgetInstruction::SetLoadedAccountsDataSizeLimit(bytes) => Some(bytes),
            _ => None,
        })
    }

    /// The requested heap size: the v1 config value, or the first
    /// `RequestHeapFrame` instruction for legacy and v0.
    pub fn get_heap_size(&self) -> Option<u32> {
        if let Some(config) = self.message.transaction_config() {
            return config.heap_size;
        }
        self.find_compute_budget(|instruction| match instruction {
            ComputeBudgetInstruction::RequestHeapFrame(bytes) => Some(bytes),
            _ => None,
        })
    }
}

fn placeholder_signatures(header: &MessageHeader) -> Vec<SignatureBytes> {
    vec![SignatureBytes::default(); header.num_required_signatures.into()]
}

/// The keys that must sign. The message must already be sanitized.
fn required_signers<'a>(header: &MessageHeader, account_keys: &'a [Pubkey]) -> &'a [Pubkey] {
    &account_keys[..header.num_required_signatures.into()]
}

fn sanitize_signature_count(header: &MessageHeader, actual: usize) -> Result<()> {
    let expected = usize::from(header.num_required_signatures);
    if actual != expected {
        return Err(SanitizeError::SignatureCountMismatch { expected, actual }.into());
    }
    Ok(())
}

fn check_size(size: usize, max: usize) -> Result<()> {
    if size > max {
        return Err(SolanaError::TransactionTooLarge { size, max });
    }
    Ok(())
}

fn is_signed(header: &MessageHeader, signatures: &[SignatureBytes]) -> bool {
    signatures.len() == usize::from(header.num_required_signatures)
        && signatures
            .iter()
            .all(|signature| !signature.is_placeholder())
}

fn sign(
    signatures: &mut Vec<SignatureBytes>,
    signers: &[Pubkey],
    message_data: &[u8],
    private_keys: &[&[u8]],
    require_all: bool,
) -> Result<()> {
    signatures.resize(signers.len(), SignatureBytes::default());
    for private_key in private_keys {
        let pubkey = Pubkey::new(get_public_key(private_key)?);
        let index = signers
            .iter()
            .position(|signer| *signer == pubkey)
            .ok_or(SolanaError::UnexpectedSigner(pubkey))?;
        signatures[index] = sign_message(private_key, message_data)?;
    }
    if require_all
        && let Some((signer, _)) = signers
            .iter()
            .zip(signatures.iter())
            .find(|(_, signature)| signature.is_placeholder())
    {
        return Err(SolanaError::MissingSigner(*signer));
    }
    Ok(())
}

fn verify(signatures: &[SignatureBytes], signers: &[Pubkey], message_data: &[u8]) -> Result<()> {
    for (signer, signature) in signers.iter().zip(signatures) {
        if signature.is_placeholder() {
            return Err(SolanaError::MissingSigner(*signer));
        }
        verify_signature(signer, message_data, signature)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionBuilder;
    use crate::crypto::hash_data;
    use crate::instructions::{compute_budget, system};
    use crate::test_utils::{LEGACY_TX, MAYAN_V0_TX, base64, key, scenarios, vectors};
    use crate::types::Instruction;
    use crate::types::{MessageV0, TransactionConfig};

    fn decode(fixture: &str) -> VersionedTransaction {
        VersionedTransaction::deserialize(&base64(fixture)).unwrap()
    }

    fn signer(label: &str) -> ([u8; 32], Pubkey) {
        let private_key = hash_data(label.as_bytes());
        (
            private_key,
            Pubkey::new(get_public_key(&private_key).unwrap()),
        )
    }

    /// Envelope + legacy message with `header`, `num_accounts` distinct keys, and a zero
    /// blockhash; callers append the instruction section.
    fn legacy_tx_prefix(num_signatures: u8, header: [u8; 3], num_accounts: u8) -> Vec<u8> {
        let mut bytes = vec![num_signatures];
        bytes.extend(std::iter::repeat_n(0u8, 64 * usize::from(num_signatures)));
        bytes.extend_from_slice(&header);
        bytes.push(num_accounts);
        for i in 0..num_accounts {
            bytes.extend_from_slice(&[i + 1; 32]);
        }
        bytes.extend_from_slice(&[0u8; 32]);
        bytes
    }

    #[test]
    fn decode_mainnet_transactions() {
        let legacy = decode(LEGACY_TX);
        assert_eq!(legacy.version(), TransactionVersion::Legacy);
        assert_eq!(legacy.signatures.len(), 1);
        assert_eq!(legacy.account_keys().len(), 22);
        assert_eq!(legacy.instructions().len(), 7);
        assert_eq!(legacy.address_table_lookups(), None);

        let v0 = decode(MAYAN_V0_TX);
        assert_eq!(v0.version(), TransactionVersion::V0);
        assert_eq!(v0.signatures.len(), 1);
        assert_eq!(v0.address_table_lookups().map(<[_]>::len), Some(3));
    }

    #[test]
    fn serialize_roundtrips_byte_for_byte() {
        for fixture in [LEGACY_TX, MAYAN_V0_TX] {
            let bytes = base64(fixture);
            let tx = VersionedTransaction::deserialize(&bytes).unwrap();
            assert_eq!(tx.serialize().unwrap(), bytes);
        }

        let legacy = decode(LEGACY_TX).into_legacy_transaction().unwrap();
        assert_eq!(legacy.serialize().unwrap(), base64(LEGACY_TX));
        assert_eq!(Transaction::deserialize(&base64(LEGACY_TX)), Ok(legacy));
        assert_eq!(
            Transaction::deserialize(&base64(MAYAN_V0_TX)),
            Err(DecodeError::UnexpectedVersion.into())
        );
    }

    #[test]
    fn compute_budget_accessors() {
        let mut legacy = decode(LEGACY_TX);
        assert_eq!(legacy.get_compute_unit_price(), Some(70_000));
        assert_eq!(legacy.get_compute_unit_limit(), Some(420_000));
        assert_eq!(legacy.get_heap_size(), None);
        assert_eq!(legacy.get_loaded_accounts_data_size_limit(), None);

        let len = legacy.serialize().unwrap().len();
        assert_eq!(legacy.set_compute_unit_price(999_999), Ok(true));
        assert_eq!(legacy.set_compute_unit_limit(500_000), Ok(true));
        assert_eq!(legacy.get_compute_unit_price(), Some(999_999));
        assert_eq!(legacy.get_compute_unit_limit(), Some(500_000));
        assert_eq!(legacy.serialize().unwrap().len(), len);

        let v0 = decode(MAYAN_V0_TX);
        assert_eq!(v0.get_compute_unit_price(), Some(71_428));
        assert_eq!(v0.get_compute_unit_limit(), Some(475_676));

        let payer = key("payer");
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            system::transfer(&payer, &key("recipient"), 1),
            compute_budget::request_heap_frame(65_536),
            compute_budget::set_loaded_accounts_data_size_limit(4096),
        ]);
        let mut tx = VersionedTransaction::from(builder.build().unwrap());
        assert_eq!(tx.get_heap_size(), Some(65_536));
        assert_eq!(tx.get_loaded_accounts_data_size_limit(), Some(4096));
        assert_eq!(tx.get_compute_unit_price(), None);
        assert_eq!(tx.set_compute_unit_price(1), Ok(false));
    }

    #[test]
    fn sign_and_verify() {
        let (payer_key, payer) = signer("payer");
        let (cosigner_key, cosigner) = signer("cosigner");
        let message = MessageV0 {
            header: MessageHeader {
                num_required_signatures: 2,
                num_readonly_signed_accounts: 1,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![payer, cosigner, key("program")],
            instructions: vec![CompiledInstruction {
                program_id_index: 2,
                accounts: vec![0, 1],
                data: vec![],
            }],
            ..MessageV0::default()
        };
        let mut tx = VersionedTransaction::new(message.into());
        assert_eq!(tx.signatures, vec![SignatureBytes::default(); 2]);
        assert!(!tx.is_signed());
        assert_eq!(tx.verify(), Err(SolanaError::MissingSigner(payer)));

        // Signatures land at the signer's index regardless of key order.
        tx.partial_sign(&[&cosigner_key]).unwrap();
        assert!(tx.signatures[0].is_placeholder());
        assert!(!tx.signatures[1].is_placeholder());
        assert_eq!(
            tx.clone().sign(&[&cosigner_key]),
            Err(SolanaError::MissingSigner(payer))
        );

        tx.sign(&[&cosigner_key, &payer_key]).unwrap();
        assert!(tx.is_signed());
        assert_eq!(tx.verify(), Ok(()));
        assert_eq!(crate::crypto::verify_transaction(&tx), Ok(()));
        assert_eq!(
            VersionedTransaction::deserialize(&tx.serialize().unwrap()),
            Ok(tx.clone())
        );

        let (stranger_key, stranger) = signer("stranger");
        assert_eq!(
            tx.partial_sign(&[&stranger_key]),
            Err(SolanaError::UnexpectedSigner(stranger))
        );

        let mut tampered = tx.clone();
        tampered.message.set_recent_blockhash([1; 32]);
        assert_eq!(tampered.verify(), Err(SolanaError::InvalidSignature));
    }

    #[test]
    fn legacy_transaction_shares_the_versioned_implementation() {
        let (payer_key, payer) = signer("payer");
        let mut builder = TransactionBuilder::new(payer, [7; 32]);
        builder.add_instruction(system::transfer(&payer, &key("recipient"), 1));
        let mut tx = builder.build().unwrap();
        assert_eq!(tx.signatures, vec![SignatureBytes::default()]);
        assert!(!tx.is_signed());

        tx.sign(&[&payer_key]).unwrap();
        assert!(tx.is_signed());
        assert_eq!(tx.verify(), Ok(()));
        assert_eq!(tx.validate_size(), Ok(()));

        let versioned = VersionedTransaction::from(tx.clone());
        assert_eq!(versioned.serialize().unwrap(), tx.serialize().unwrap());
        assert_eq!(
            versioned.serialize_message().unwrap(),
            tx.message_data().unwrap()
        );
        assert_eq!(versioned.verify(), Ok(()));
    }

    #[test]
    fn validate_size_limit() {
        let mut tx = decode(LEGACY_TX);
        assert_eq!(tx.validate_size(), Ok(()));
        let VersionedMessage::Legacy(message) = &mut tx.message else {
            unreachable!()
        };
        message.instructions[0].data = vec![0; 1024];
        let size = tx.serialize().unwrap().len();
        assert_eq!(
            tx.validate_size(),
            Err(SolanaError::TransactionTooLarge {
                size,
                max: MAX_TRANSACTION_SIZE
            })
        );
    }

    #[test]
    fn uses_durable_nonce() {
        let payer = key("payer");
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            system::advance_nonce_account(&key("nonce"), &payer),
            system::transfer(&payer, &key("recipient"), 1),
        ]);
        let tx = VersionedTransaction::from(builder.build().unwrap());
        assert!(tx.uses_durable_nonce());
        assert!(!decode(LEGACY_TX).uses_durable_nonce());
    }

    #[test]
    fn v1_transactions() {
        let tx = VersionedTransaction::deserialize(&vectors::V1_COMPLEX_TX).unwrap();
        assert_eq!(tx.version(), TransactionVersion::V1);
        assert_eq!(tx.max_size(), 4096);
        assert_eq!(tx.signatures.len(), 3);
        assert_eq!(tx.address_table_lookups(), None);
        assert_eq!(
            tx.message.transaction_config(),
            Some(&scenarios::full_config())
        );
        assert_eq!(tx.verify(), Ok(()));

        // Config values, never Compute Budget instructions.
        assert_eq!(tx.priority_fee_lamports(), Some(12_345));
        assert_eq!(tx.get_compute_unit_limit(), Some(300_000));
        assert_eq!(tx.get_loaded_accounts_data_size_limit(), Some(65_536));
        assert_eq!(tx.get_heap_size(), Some(65_536));
        assert_eq!(tx.get_compute_unit_price(), None);

        let mut tx = tx;
        assert_eq!(
            tx.set_compute_unit_price(1),
            Err(SolanaError::UnsupportedVersion)
        );
        assert_eq!(tx.set_compute_unit_limit(7), Ok(true));
        assert_eq!(tx.set_priority_fee_lamports(9), Ok(()));
        assert_eq!(
            tx.message.transaction_config(),
            Some(
                &scenarios::full_config()
                    .with_compute_unit_limit(7)
                    .with_priority_fee(9)
            )
        );
        assert_eq!(tx.verify(), Err(SolanaError::InvalidSignature));

        let partial = VersionedTransaction::deserialize(&vectors::V1_NONCE_TX).unwrap();
        assert!(partial.uses_durable_nonce());
        assert_eq!(partial.priority_fee_lamports(), None);
        assert_eq!(partial.get_loaded_accounts_data_size_limit(), None);
        assert_eq!(partial.get_heap_size(), Some(32 * 1024));

        let mut legacy = decode(LEGACY_TX);
        assert_eq!(legacy.priority_fee_lamports(), None);
        assert_eq!(
            legacy.set_priority_fee_lamports(1),
            Err(SolanaError::UnsupportedVersion)
        );
    }

    #[test]
    fn v1_ignores_compute_budget_instructions() {
        let mut builder = TransactionBuilder::new(key("payer"), [0; 32]);
        builder.add_instructions(scenarios::complex());
        let mut tx = builder.build_v1(TransactionConfig::new()).unwrap();
        assert_eq!(tx.get_compute_unit_limit(), None);
        assert_eq!(tx.get_compute_unit_price(), None);
        assert_eq!(tx.set_compute_unit_limit(1), Ok(true));
        assert_eq!(
            tx.instructions()[0].data,
            compute_budget::set_compute_unit_limit(200_000).data
        );
    }

    #[test]
    fn v1_wire_errors() {
        for tx in [
            &vectors::V1_SIMPLE_TX[..],
            &vectors::V1_COMPLEX_TX,
            &vectors::V1_NONCE_TX,
            &vectors::V1_FEE_ONLY_TX,
        ] {
            for len in 0..tx.len() {
                assert!(
                    VersionedTransaction::deserialize(&tx[..len]).is_err(),
                    "truncated to {len} bytes"
                );
            }
            assert_eq!(
                VersionedTransaction::deserialize(&[tx, &[0]].concat()),
                Err(DecodeError::TrailingBytes.into())
            );
        }

        let mut tx = VersionedTransaction::deserialize(&vectors::V1_SIMPLE_TX).unwrap();
        tx.signatures.push(SignatureBytes::default());
        assert_eq!(
            tx.serialize(),
            Err(crate::EncodeError::SignatureCountMismatch {
                expected: 1,
                actual: 2
            }
            .into())
        );

        // Half of the priority-fee bit pair.
        let mut bytes = vectors::V1_SIMPLE_TX;
        bytes[4] = 0b01;
        assert_eq!(
            VersionedTransaction::deserialize(&bytes),
            Err(DecodeError::InvalidConfigMask(1).into())
        );
    }

    #[test]
    fn deserialize_enforces_version_size_limits() {
        let payer = key("payer");
        let big_instruction = |len| Instruction {
            program_id: key("program"),
            accounts: vec![],
            data: vec![0; len],
        };
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instruction(big_instruction(1200));
        let legacy = VersionedTransaction::from(builder.build().unwrap());
        let bytes = legacy.serialize().unwrap();
        assert!(bytes.len() > MAX_TRANSACTION_SIZE);
        assert_eq!(
            VersionedTransaction::deserialize(&bytes),
            Err(SolanaError::TransactionTooLarge {
                size: bytes.len(),
                max: MAX_TRANSACTION_SIZE
            })
        );

        // The same message fits in v1, up to 4096 bytes.
        let v1 = builder.build_v1(TransactionConfig::new()).unwrap();
        assert_eq!(v1.validate_size(), Ok(()));
        let bytes = v1.serialize().unwrap();
        assert_eq!(VersionedTransaction::deserialize(&bytes), Ok(v1));

        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instruction(big_instruction(4000));
        let v1 = builder.build_v1(TransactionConfig::new()).unwrap();
        let bytes = v1.serialize().unwrap();
        assert_eq!(
            VersionedTransaction::deserialize(&bytes),
            Err(SolanaError::TransactionTooLarge {
                size: bytes.len(),
                max: 4096
            })
        );
    }

    #[test]
    fn deserialize_accepts_minimal_transactions() {
        let mut legacy = legacy_tx_prefix(1, [1, 0, 1], 2);
        legacy.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert!(VersionedTransaction::deserialize(&legacy).is_ok());

        let mut v0 = legacy_tx_prefix(1, [1, 0, 1], 2);
        v0.insert(65, 0x80);
        v0.extend_from_slice(&[1, 1, 1, 0, 0]);
        v0.push(0); // address table lookup count
        assert!(VersionedTransaction::deserialize(&v0).is_ok());
    }

    #[test]
    fn deserialize_rejects_truncation_and_trailing_bytes() {
        for fixture in [LEGACY_TX, MAYAN_V0_TX] {
            let data = base64(fixture);
            for len in 0..data.len() {
                assert!(
                    VersionedTransaction::deserialize(&data[..len]).is_err(),
                    "truncated to {len} bytes"
                );
            }
            let padded = [&data[..], &[0]].concat();
            assert_eq!(
                VersionedTransaction::deserialize(&padded),
                Err(DecodeError::TrailingBytes.into())
            );
        }
    }

    #[test]
    fn deserialize_requires_v0_lookup_count() {
        let mut tx = decode(MAYAN_V0_TX);
        let VersionedMessage::V0(message) = &mut tx.message else {
            unreachable!()
        };
        message.address_table_lookups.clear();
        let mut bytes = tx.serialize().unwrap();
        assert_eq!(bytes.pop(), Some(0));
        assert_eq!(
            VersionedTransaction::deserialize(&bytes),
            Err(DecodeError::UnexpectedEof.into())
        );
    }

    #[test]
    fn deserialize_rejects_unknown_discriminators() {
        let mut message = legacy_tx_prefix(0, [1, 0, 1], 2);
        message.extend_from_slice(&[1, 1, 1, 0, 0, 0]);
        // A bare v0 message, an unknown version, and the off-chain message prefix.
        for first in [0x80, 0x82, 0xff] {
            message[0] = first;
            assert_eq!(
                VersionedTransaction::deserialize(&message),
                Err(DecodeError::InvalidTransactionDiscriminator(first).into())
            );
        }

        let mut unknown_version = legacy_tx_prefix(1, [1, 0, 1], 2);
        unknown_version.insert(65, 0x82);
        unknown_version.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_eq!(
            VersionedTransaction::deserialize(&unknown_version),
            Err(DecodeError::UnsupportedMessageVersion(2).into())
        );

        // A v1 message is only valid in the v1 envelope.
        let (message, signature) = vectors::V1_SIMPLE_TX.split_at(vectors::V1_SIMPLE_TX.len() - 64);
        let v1_in_legacy_envelope = [&[1], signature, message].concat();
        assert_eq!(
            VersionedTransaction::deserialize(&v1_in_legacy_envelope),
            Err(DecodeError::UnexpectedVersion.into())
        );
    }

    #[test]
    fn deserialize_rejects_non_canonical_lengths() {
        // Account count 2 encoded as the alias [0x82, 0x00].
        let mut bytes = legacy_tx_prefix(1, [1, 0, 1], 2);
        bytes.splice(68..69, [0x82, 0x00]);
        bytes.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_eq!(
            VersionedTransaction::deserialize(&bytes),
            Err(DecodeError::NonCanonicalShortU16.into())
        );
    }

    #[test]
    fn deserialize_sanitizes() {
        use SanitizeError::*;
        let cases: [([u8; 3], [u8; 5], SanitizeError); 6] = [
            ([1, 0, 5], [1, 1, 1, 0, 0], NotEnoughAccountKeys),
            ([1, 1, 1], [1, 1, 1, 0, 0], NoWritableFeePayer),
            ([0, 0, 1], [1, 1, 1, 0, 0], NoWritableFeePayer),
            ([1, 0, 1], [1, 0, 1, 0, 0], InvalidProgramIndex),
            ([1, 0, 1], [1, 2, 1, 0, 0], InvalidProgramIndex),
            ([1, 0, 1], [1, 1, 1, 2, 0], InvalidAccountIndex),
        ];
        for (header, instruction, expected) in cases {
            let mut bytes = legacy_tx_prefix(header[0], header, 2);
            bytes.extend_from_slice(&instruction);
            assert_eq!(
                VersionedTransaction::deserialize(&bytes),
                Err(expected.into())
            );
        }

        let mut extra_signature = legacy_tx_prefix(2, [1, 0, 1], 2);
        extra_signature.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_eq!(
            VersionedTransaction::deserialize(&extra_signature),
            Err(SignatureCountMismatch {
                expected: 1,
                actual: 2
            }
            .into())
        );

        // 60,000 instructions cannot fit in the remaining input.
        let mut huge_instruction_count = legacy_tx_prefix(1, [1, 0, 1], 2);
        huge_instruction_count.extend_from_slice(&[0xe0, 0xd4, 0x03]);
        assert_eq!(
            VersionedTransaction::deserialize(&huge_instruction_count),
            Err(DecodeError::UnexpectedEof.into())
        );
    }
}
