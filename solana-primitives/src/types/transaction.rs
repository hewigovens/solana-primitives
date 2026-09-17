#[cfg(feature = "signing")]
use crate::crypto::{get_public_key, sign_message, verify_signature};
use crate::error::{Result, SanitizeError, SolanaError};
use crate::instructions::compute_budget::{
    ComputeBudgetInstruction, parse_compute_budget_requests,
};
use crate::instructions::program_ids::{compute_budget_program, system_program};
use crate::instructions::system::is_advance_nonce_instruction_data;
use crate::types::{
    CompiledInstruction, MAX_TRANSACTION_SIZE, MessageAddressTableLookup, MessageHeader, Pubkey,
    SignatureBytes, VersionedMessage, v1,
};
use crate::wire;

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

/// A legacy, v0, or v1 transaction.
///
/// Legacy and v0 serialize as `compact-u16 signature count || signatures ||
/// message`; v1 as `message || signatures`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VersionedTransaction {
    /// One signature per required signer, in account key order.
    pub signatures: Vec<SignatureBytes>,
    /// The message.
    pub message: VersionedMessage,
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

    /// The signatures.
    pub fn signatures(&self) -> &[SignatureBytes] {
        &self.signatures
    }

    /// The message header.
    pub fn header(&self) -> &MessageHeader {
        self.message.header()
    }

    /// How many leading account keys must sign.
    pub fn num_required_signatures(&self) -> u8 {
        self.header().num_required_signatures
    }

    /// How many of the signing keys are read-only.
    pub fn num_readonly_signed_accounts(&self) -> u8 {
        self.header().num_readonly_signed_accounts
    }

    /// How many of the non-signing keys are read-only.
    pub fn num_readonly_unsigned_accounts(&self) -> u8 {
        self.header().num_readonly_unsigned_accounts
    }

    /// Static account keys (looked-up keys are not included).
    pub fn account_keys(&self) -> &[Pubkey] {
        self.message.static_account_keys()
    }

    /// The recent blockhash, or the nonce value for a durable-nonce transaction.
    pub fn recent_blockhash(&self) -> &[u8; 32] {
        self.message.recent_blockhash()
    }

    /// The compiled instructions, in execution order.
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

    /// Place a signature made elsewhere (for example by a hardware wallet) in
    /// every slot `signer` occupies. The signature is not verified.
    ///
    /// Fails if `signer` is not a required signer.
    pub fn add_signature(&mut self, signer: &Pubkey, signature: SignatureBytes) -> Result<()> {
        self.message.sanitize()?;
        let signers = required_signers(self.message.header(), self.message.static_account_keys());
        place_signature(&mut self.signatures, signers, signer, signature)
    }

    /// Sign with every required signer.
    ///
    /// Each key must belong to a required signer; its signature is placed at
    /// that signer's index. Fails if any required signature is still missing.
    #[cfg(feature = "signing")]
    pub fn sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.sign_with(private_keys)?;
        let signers = required_signers(self.message.header(), self.message.static_account_keys());
        match signers
            .iter()
            .zip(&self.signatures)
            .find(|(_, signature)| signature.is_placeholder())
        {
            Some((signer, _)) => Err(SolanaError::MissingSigner(*signer)),
            None => Ok(()),
        }
    }

    /// Sign with some of the required signers, leaving other signatures as they are.
    ///
    /// Each key must belong to a required signer.
    #[cfg(feature = "signing")]
    pub fn partial_sign(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.sign_with(private_keys)
    }

    #[cfg(feature = "signing")]
    fn sign_with(&mut self, private_keys: &[&[u8]]) -> Result<()> {
        self.message.sanitize()?;
        let message_data = self.serialize_message()?;
        let signers = required_signers(self.message.header(), self.message.static_account_keys());
        for private_key in private_keys {
            let signer = Pubkey::new(get_public_key(private_key)?);
            let signature = sign_message(private_key, &message_data)?;
            place_signature(&mut self.signatures, signers, &signer, signature)?;
        }
        Ok(())
    }

    /// Whether every required signature is present (not verified).
    pub fn is_signed(&self) -> bool {
        self.signatures.len() == usize::from(self.num_required_signatures())
            && self
                .signatures
                .iter()
                .all(|signature| !signature.is_placeholder())
    }

    /// Sanitize and verify every required signature.
    #[cfg(feature = "signing")]
    pub fn verify(&self) -> Result<()> {
        self.sanitize()?;
        let message_data = self.serialize_message()?;
        let signers = required_signers(self.header(), self.account_keys());
        for (signer, signature) in signers.iter().zip(&self.signatures) {
            if signature.is_placeholder() {
                return Err(SolanaError::MissingSigner(*signer));
            }
            verify_signature(signer, &message_data, signature)?;
        }
        Ok(())
    }

    fn program_id(&self, instruction: &CompiledInstruction) -> Option<Pubkey> {
        self.account_keys()
            .get(usize::from(instruction.program_id_index))
            .copied()
    }

    /// Legacy/v0 Compute Budget requests with their instruction positions, or `None`
    /// when the runtime would reject them. v1 has none: its runtime ignores the
    /// instructions.
    fn compute_budget_requests(&self) -> Option<Vec<(usize, ComputeBudgetInstruction)>> {
        if self.version() == TransactionVersion::V1 {
            return Some(Vec::new());
        }
        let program_id = compute_budget_program();
        let instructions = self
            .instructions()
            .iter()
            .enumerate()
            .filter(|(_, instruction)| self.program_id(instruction) == Some(program_id))
            .map(|(index, instruction)| (index, instruction.data.as_slice()));
        parse_compute_budget_requests(instructions)
    }

    fn find_compute_budget<T>(
        &self,
        select: impl Fn(ComputeBudgetInstruction) -> Option<T>,
    ) -> Option<T> {
        self.compute_budget_requests()?
            .into_iter()
            .find_map(|(_, request)| select(request))
    }

    /// Overwrite the value bytes (after the discriminant) of the request accepted by
    /// `select`, keeping the instruction length unchanged. Returns `false` if there
    /// is no such valid request.
    fn replace_compute_budget_value(
        &mut self,
        select: impl Fn(ComputeBudgetInstruction) -> bool,
        value: &[u8],
    ) -> bool {
        let Some((index, _)) = self
            .compute_budget_requests()
            .and_then(|requests| requests.into_iter().find(|(_, request)| select(*request)))
        else {
            return false;
        };
        let target = &mut self.message.instructions_mut()[index].data[1..=value.len()];
        if target != value {
            target.copy_from_slice(value);
            self.clear_signatures();
        }
        true
    }

    /// Update the v1 config, clearing signatures if it changes.
    fn update_v1_config(&mut self, update: impl FnOnce(&mut v1::TransactionConfig)) -> Result<()> {
        let VersionedMessage::V1(message) = &mut self.message else {
            return Err(SolanaError::UnsupportedVersion);
        };
        let before = message.config;
        update(&mut message.config);
        if message.config != before {
            self.clear_signatures();
        }
        Ok(())
    }

    /// Reset every signature to the placeholder after the signed bytes change.
    fn clear_signatures(&mut self) {
        self.signatures = placeholder_signatures(self.header());
    }

    /// Replace the recent blockhash (the v1 lifetime specifier), clearing signatures
    /// if it changes.
    pub fn set_recent_blockhash(&mut self, recent_blockhash: [u8; 32]) {
        if *self.recent_blockhash() != recent_blockhash {
            self.message.set_recent_blockhash(recent_blockhash);
            self.clear_signatures();
        }
    }

    /// The legacy/v0 compute unit price in micro-lamports per compute unit.
    ///
    /// Always `None` for v1, which pays a total fee instead; see
    /// [`VersionedTransaction::priority_fee_lamports`].
    pub fn get_compute_unit_price(&self) -> Option<u64> {
        self.find_compute_budget(|request| match request {
            ComputeBudgetInstruction::SetComputeUnitPrice(micro_lamports) => Some(micro_lamports),
            _ => None,
        })
    }

    /// Overwrite the `SetComputeUnitPrice` value; returns `false` if there is none.
    /// Signatures are cleared if the value changes.
    ///
    /// Fails with [`SolanaError::UnsupportedVersion`] for v1; use
    /// [`VersionedTransaction::set_priority_fee_lamports`].
    pub fn set_compute_unit_price(&mut self, micro_lamports: u64) -> Result<bool> {
        if self.version() == TransactionVersion::V1 {
            return Err(SolanaError::UnsupportedVersion);
        }
        Ok(self.replace_compute_budget_value(
            |request| matches!(request, ComputeBudgetInstruction::SetComputeUnitPrice(_)),
            &micro_lamports.to_le_bytes(),
        ))
    }

    /// The v1 total priority fee in lamports; `None` if unset or not v1.
    pub fn priority_fee_lamports(&self) -> Option<u64> {
        self.message.transaction_config()?.priority_fee
    }

    /// Set the v1 total priority fee in lamports. Signatures are cleared if it changes.
    ///
    /// Fails with [`SolanaError::UnsupportedVersion`] for legacy and v0; use
    /// [`VersionedTransaction::set_compute_unit_price`].
    pub fn set_priority_fee_lamports(&mut self, lamports: u64) -> Result<()> {
        self.update_v1_config(|config| config.priority_fee = Some(lamports))
    }

    /// The requested compute unit limit: the v1 config value, or the legacy/v0
    /// `SetComputeUnitLimit` instruction.
    pub fn get_compute_unit_limit(&self) -> Option<u32> {
        if let Some(config) = self.message.transaction_config() {
            return config.compute_unit_limit;
        }
        self.find_compute_budget(|request| match request {
            ComputeBudgetInstruction::SetComputeUnitLimit(units) => Some(units),
            _ => None,
        })
    }

    /// Set the compute unit limit; returns `false` if a legacy/v0 transaction has no
    /// `SetComputeUnitLimit` instruction to overwrite. v1 always sets its config.
    /// Signatures are cleared if the value changes.
    pub fn set_compute_unit_limit(&mut self, units: u32) -> Result<bool> {
        if self.version() == TransactionVersion::V1 {
            self.update_v1_config(|config| config.compute_unit_limit = Some(units))?;
            return Ok(true);
        }
        Ok(self.replace_compute_budget_value(
            |request| matches!(request, ComputeBudgetInstruction::SetComputeUnitLimit(_)),
            &units.to_le_bytes(),
        ))
    }

    /// The requested loaded accounts data size limit: the v1 config value, or the
    /// legacy/v0 `SetLoadedAccountsDataSizeLimit` instruction.
    pub fn get_loaded_accounts_data_size_limit(&self) -> Option<u32> {
        if let Some(config) = self.message.transaction_config() {
            return config.loaded_accounts_data_size_limit;
        }
        self.find_compute_budget(|request| match request {
            ComputeBudgetInstruction::SetLoadedAccountsDataSizeLimit(bytes) => Some(bytes),
            _ => None,
        })
    }

    /// The requested heap size: the v1 config value, or the legacy/v0
    /// `RequestHeapFrame` instruction.
    pub fn get_heap_size(&self) -> Option<u32> {
        if let Some(config) = self.message.transaction_config() {
            return config.heap_size;
        }
        self.find_compute_budget(|request| match request {
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

/// Put `signature` in each of `signer`'s slots, sizing `signatures` to the signer count.
fn place_signature(
    signatures: &mut Vec<SignatureBytes>,
    signers: &[Pubkey],
    signer: &Pubkey,
    signature: SignatureBytes,
) -> Result<()> {
    if !signers.contains(signer) {
        return Err(SolanaError::UnexpectedSigner(*signer));
    }
    signatures.resize(signers.len(), SignatureBytes::default());
    // A key may occupy several signer slots; each needs the signature.
    for (key, slot) in signers.iter().zip(signatures.iter_mut()) {
        if key == signer {
            *slot = signature;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionBuilder;
    use crate::error::DecodeError;
    use crate::instructions::{compute_budget, system};
    use crate::test_utils::{LEGACY_TX, MAYAN_V0_TX, base64, key, scenarios, signer, vectors};
    use crate::types::{Instruction, Message, TransactionConfig};

    fn decode(fixture: &str) -> VersionedTransaction {
        VersionedTransaction::deserialize(&base64(fixture)).unwrap()
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
        let mut tx = builder.build().unwrap();
        assert_eq!(tx.get_heap_size(), Some(65_536));
        assert_eq!(tx.get_loaded_accounts_data_size_limit(), Some(4096));
        assert_eq!(tx.get_compute_unit_price(), None);
        assert_eq!(tx.set_compute_unit_price(1), Ok(false));
    }

    #[test]
    fn zero_loaded_accounts_limit_is_version_specific() {
        let payer = key("payer");
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            compute_budget::set_compute_unit_price(5),
            compute_budget::set_loaded_accounts_data_size_limit(0),
        ]);

        // The runtime fails a legacy/v0 transaction that requests a zero limit.
        let mut legacy = builder.build().unwrap();
        assert_eq!(legacy.get_loaded_accounts_data_size_limit(), None);
        assert_eq!(legacy.get_compute_unit_price(), None);
        assert_eq!(legacy.set_compute_unit_price(1), Ok(false));

        // SIMD-0385 allows 0 in a v1 config (and uses it when the value is unset).
        let config = TransactionConfig::new().with_loaded_accounts_data_size_limit(0);
        let v1 = builder.build_v1(config).unwrap();
        let v1 = VersionedTransaction::deserialize(&v1.serialize().unwrap()).unwrap();
        assert_eq!(v1.get_loaded_accounts_data_size_limit(), Some(0));
    }

    fn duplicated_signer_message() -> Message {
        let payer = signer("payer").pubkey;
        // Sanitize allows a repeated key (the runtime rejects it later).
        Message {
            header: MessageHeader {
                num_required_signatures: 2,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            account_keys: vec![payer, payer, key("program")],
            instructions: vec![CompiledInstruction {
                program_id_index: 2,
                accounts: vec![],
                data: vec![],
            }],
            ..Message::default()
        }
    }

    #[test]
    fn add_signature_places_external_signatures() {
        let (payer, cosigner) = (signer("payer").pubkey, signer("cosigner").pubkey);
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            system::transfer(&payer, &key("recipient"), 1),
            system::transfer(&cosigner, &key("recipient"), 1),
        ]);
        let mut tx = builder.build().unwrap();
        let (a, b) = (SignatureBytes::new([1; 64]), SignatureBytes::new([2; 64]));

        tx.add_signature(&cosigner, b).unwrap();
        assert_eq!(tx.signatures, vec![SignatureBytes::default(), b]);
        assert!(!tx.is_signed());
        tx.add_signature(&payer, a).unwrap();
        assert_eq!(tx.signatures, vec![a, b]);
        assert!(tx.is_signed());

        let stranger = key("stranger");
        assert_eq!(
            tx.add_signature(&stranger, a),
            Err(SolanaError::UnexpectedSigner(stranger))
        );

        // A key in several signer slots gets the signature in each.
        let mut duplicated = VersionedTransaction::new(duplicated_signer_message().into());
        duplicated.add_signature(&payer, a).unwrap();
        assert_eq!(duplicated.signatures, vec![a, a]);
    }

    #[test]
    fn setters_clear_stale_signatures() {
        let (payer, cosigner) = (signer("payer").pubkey, signer("cosigner").pubkey);
        let transfer = |from: &Pubkey| system::transfer(from, &key("recipient"), 1);
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            compute_budget::set_compute_unit_price(1),
            compute_budget::set_compute_unit_limit(1),
            transfer(&payer),
            transfer(&cosigner),
        ]);
        let sign_all = |tx: &mut VersionedTransaction| {
            tx.add_signature(&payer, SignatureBytes::new([1; 64]))
                .unwrap();
            tx.add_signature(&cosigner, SignatureBytes::new([2; 64]))
                .unwrap();
            assert!(tx.is_signed());
        };

        let mut legacy = builder.build().unwrap();
        sign_all(&mut legacy);
        // Writing the current value keeps the signatures.
        assert_eq!(legacy.set_compute_unit_price(1), Ok(true));
        assert!(legacy.is_signed());
        assert_eq!(legacy.set_compute_unit_price(2), Ok(true));
        assert!(!legacy.is_signed());
        assert!(legacy.signatures.iter().all(SignatureBytes::is_placeholder));
        sign_all(&mut legacy);
        assert_eq!(legacy.set_compute_unit_limit(2), Ok(true));
        assert!(!legacy.is_signed());
        sign_all(&mut legacy);
        legacy.set_recent_blockhash([0; 32]);
        assert!(legacy.is_signed());
        legacy.set_recent_blockhash([1; 32]);
        assert!(!legacy.is_signed());

        let mut v1 = builder.build_v1(TransactionConfig::new()).unwrap();
        sign_all(&mut v1);
        v1.set_priority_fee_lamports(5).unwrap();
        assert!(!v1.is_signed());
        sign_all(&mut v1);
        v1.set_priority_fee_lamports(5).unwrap();
        assert!(v1.is_signed());
        v1.set_compute_unit_limit(9).unwrap();
        assert!(!v1.is_signed());
    }

    #[cfg(feature = "signing")]
    mod signing {
        use super::*;
        use crate::crypto::{get_public_key, hash_data};
        use crate::types::MessageV0;

        /// `sha256(label)` as a private key, with its public key.
        fn keypair(label: &str) -> ([u8; 32], Pubkey) {
            let private_key = hash_data(label.as_bytes());
            let pubkey = Pubkey::new(get_public_key(&private_key).unwrap());
            (private_key, pubkey)
        }

        #[test]
        fn sign_and_verify() {
            let (payer_key, payer) = keypair("payer");
            let (cosigner_key, cosigner) = keypair("cosigner");
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

            // An externally made signature verifies the same way.
            let mut external = VersionedTransaction::new(tx.message.clone());
            let message_data = external.serialize_message().unwrap();
            for key in [payer_key, cosigner_key] {
                let pubkey = Pubkey::new(get_public_key(&key).unwrap());
                let signature = crate::crypto::sign_message(&key, &message_data).unwrap();
                external.add_signature(&pubkey, signature).unwrap();
            }
            assert_eq!(external, tx);

            let (stranger_key, stranger) = keypair("stranger");
            assert_eq!(
                tx.partial_sign(&[&stranger_key]),
                Err(SolanaError::UnexpectedSigner(stranger))
            );

            let mut tampered = tx.clone();
            tampered.message.set_recent_blockhash([1; 32]);
            assert_eq!(tampered.verify(), Err(SolanaError::InvalidSignature));
        }

        #[test]
        fn duplicate_signer_keys_get_every_slot() {
            let mut tx = VersionedTransaction::new(duplicated_signer_message().into());
            tx.sign(&[&signer("payer").private_key]).unwrap();
            assert_eq!(tx.signatures[0], tx.signatures[1]);
            assert_eq!(tx.verify(), Ok(()));
        }

        #[test]
        fn resigning_after_a_change_needs_every_signer() {
            let (payer_key, payer) = keypair("payer");
            let (cosigner_key, cosigner) = keypair("cosigner");
            let mut builder = TransactionBuilder::new(payer, [0; 32]);
            builder.add_instructions([
                compute_budget::set_compute_unit_price(1),
                system::transfer(&cosigner, &key("recipient"), 1),
            ]);
            let mut tx = builder.build().unwrap();
            tx.sign(&[&payer_key, &cosigner_key]).unwrap();
            assert_eq!(tx.set_compute_unit_price(2), Ok(true));
            assert_eq!(
                tx.sign(&[&payer_key]),
                Err(SolanaError::MissingSigner(cosigner))
            );
            tx.sign(&[&payer_key, &cosigner_key]).unwrap();
            assert_eq!(tx.verify(), Ok(()));
        }

        #[test]
        fn builder_transactions_sign_and_verify() {
            let (payer_key, payer) = keypair("payer");
            let mut builder = TransactionBuilder::new(payer, [7; 32]);
            builder.add_instruction(system::transfer(&payer, &key("recipient"), 1));
            for mut tx in [
                builder.build().unwrap(),
                builder.build_v0(&[]).unwrap(),
                builder.build_v1(scenarios::full_config()).unwrap(),
            ] {
                assert!(!tx.is_signed());
                tx.sign(&[&payer_key]).unwrap();
                assert!(tx.is_signed());
                assert_eq!(tx.verify(), Ok(()));
                assert_eq!(tx.validate_size(), Ok(()));
            }
        }
    }

    #[test]
    fn compute_budget_accessors_follow_the_runtime() {
        let payer = key("payer");
        let retired = Instruction {
            data: vec![0],
            ..compute_budget::set_compute_unit_limit(0)
        };
        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            retired,
            compute_budget::set_compute_unit_limit(5),
            compute_budget::set_compute_unit_price(7),
        ]);
        // The runtime fails the transaction, so nothing is requested.
        let mut tx = builder.build().unwrap();
        assert_eq!(tx.get_compute_unit_limit(), None);
        assert_eq!(tx.get_compute_unit_price(), None);
        assert_eq!(tx.set_compute_unit_price(8), Ok(false));

        let mut builder = TransactionBuilder::new(payer, [0; 32]);
        builder.add_instructions([
            compute_budget::set_compute_unit_limit(5),
            compute_budget::set_compute_unit_limit(6),
        ]);
        let tx = builder.build().unwrap();
        assert_eq!(tx.get_compute_unit_limit(), None);
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
        let tx = builder.build().unwrap();
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
        #[cfg(feature = "signing")]
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
        // The old signatures no longer cover the message, so they are cleared.
        assert!(tx.signatures.iter().all(SignatureBytes::is_placeholder));

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
        let legacy = builder.build().unwrap();
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

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrips_every_version() {
        for bytes in [
            &base64(LEGACY_TX)[..],
            &base64(MAYAN_V0_TX),
            &vectors::V1_COMPLEX_TX,
        ] {
            let tx = VersionedTransaction::deserialize(bytes).unwrap();
            let json = serde_json::to_string(&tx).unwrap();
            assert_eq!(
                serde_json::from_str::<VersionedTransaction>(&json).unwrap(),
                tx
            );
        }
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
        let legacy_and_v0 = [base64(LEGACY_TX), base64(MAYAN_V0_TX)];
        let v1 = [
            &vectors::V1_SIMPLE_TX[..],
            &vectors::V1_COMPLEX_TX,
            &vectors::V1_NONCE_TX,
            &vectors::V1_FEE_ONLY_TX,
        ];
        for data in legacy_and_v0.iter().map(Vec::as_slice).chain(v1) {
            for len in 0..data.len() {
                assert!(
                    VersionedTransaction::deserialize(&data[..len]).is_err(),
                    "truncated to {len} bytes"
                );
            }
            assert_eq!(
                VersionedTransaction::deserialize(&[data, &[0]].concat()),
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
            Err(DecodeError::NonCanonicalCompactU16.into())
        );
    }

    #[test]
    fn deserialize_sanitizes() {
        // The rules themselves are covered by the message tests; this checks that
        // decoding applies them.
        let mut no_writable_fee_payer = legacy_tx_prefix(1, [1, 1, 1], 2);
        no_writable_fee_payer.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_eq!(
            VersionedTransaction::deserialize(&no_writable_fee_payer),
            Err(SanitizeError::NoWritableFeePayer.into())
        );

        let mut extra_signature = legacy_tx_prefix(2, [1, 0, 1], 2);
        extra_signature.extend_from_slice(&[1, 1, 1, 0, 0]);
        assert_eq!(
            VersionedTransaction::deserialize(&extra_signature),
            Err(SanitizeError::SignatureCountMismatch {
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
