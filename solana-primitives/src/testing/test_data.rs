//! Test data generators

#[cfg(feature = "testing")]
use crate::{
    crypto::Keypair,
    types::{
        AccountMeta, CompiledInstruction, Instruction, LegacyMessage, Message, MessageHeader,
        Pubkey, SignatureBytes, Transaction, VersionedTransaction,
    },
};
#[cfg(feature = "testing")]
use rand::{thread_rng, Rng};

/// Test data generator utilities
#[cfg(feature = "testing")]
pub struct TestDataGenerator;

#[cfg(feature = "testing")]
impl TestDataGenerator {
    /// Generate a random keypair
    pub fn random_keypair() -> Keypair {
        Keypair::generate()
    }

    /// Generate a random public key
    pub fn random_pubkey() -> Pubkey {
        Self::random_keypair().pubkey()
    }

    /// Generate a random signature
    pub fn random_signature() -> SignatureBytes {
        let mut rng = thread_rng();
        let mut bytes = [0u8; 64];
        rng.fill(&mut bytes);
        SignatureBytes::new(bytes)
    }

    /// Generate random account data
    pub fn sample_account_data(size: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        (0..size).map(|_| rng.gen()).collect()
    }

    /// Generate a sample instruction
    pub fn sample_instruction() -> Instruction {
        Instruction {
            program_id: Self::random_pubkey(),
            accounts: vec![
                AccountMeta {
                    pubkey: Self::random_pubkey(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: Self::random_pubkey(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: Self::sample_account_data(32),
        }
    }

    /// Generate a sample compiled instruction
    pub fn sample_compiled_instruction() -> CompiledInstruction {
        CompiledInstruction {
            program_id_index: 1,
            accounts: vec![0, 2],
            data: Self::sample_account_data(16),
        }
    }

    /// Generate a sample message
    pub fn sample_message() -> Message {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        };
        let account_keys = vec![Self::random_pubkey(), Self::random_pubkey()];
        let recent_blockhash = Self::random_blockhash();
        let instructions = vec![Self::sample_compiled_instruction()];

        Message {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        }
    }

    /// Generate a sample legacy message
    pub fn sample_legacy_message() -> LegacyMessage {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        };
        let account_keys = vec![Self::random_pubkey(), Self::random_pubkey()];
        let recent_blockhash = Self::random_blockhash();
        let instructions = vec![Self::sample_compiled_instruction()];

        LegacyMessage {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        }
    }

    /// Generate a sample transaction
    pub fn sample_transaction() -> Transaction {
        let message = Self::sample_message();
        let signatures = vec![Self::random_signature()];

        Transaction { signatures, message }
    }

    /// Generate a sample versioned transaction
    pub fn sample_versioned_transaction() -> VersionedTransaction {
        let message = Self::sample_legacy_message();
        let signatures = vec![Self::random_signature()];

        VersionedTransaction::Legacy { signatures, message }
    }

    /// Generate a random blockhash
    pub fn random_blockhash() -> [u8; 32] {
        let mut rng = thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        bytes
    }

    /// Generate test keypairs for common scenarios
    pub fn test_keypairs(count: usize) -> Vec<Keypair> {
        (0..count).map(|_| Self::random_keypair()).collect()
    }

    /// Generate test pubkeys for common scenarios
    pub fn test_pubkeys(count: usize) -> Vec<Pubkey> {
        (0..count).map(|_| Self::random_pubkey()).collect()
    }

    /// Generate a deterministic keypair from a seed
    pub fn keypair_from_seed(seed: u8) -> Keypair {
        let seed_bytes = [seed; 32];
        Keypair::from_seed(&seed_bytes).expect("Valid seed")
    }

    /// Generate a deterministic pubkey from a seed
    pub fn pubkey_from_seed(seed: u8) -> Pubkey {
        Pubkey::new([seed; 32])
    }

    /// Generate test account metas
    pub fn test_account_metas(count: usize) -> Vec<AccountMeta> {
        (0..count)
            .map(|i| AccountMeta {
                pubkey: Self::pubkey_from_seed(i as u8),
                is_signer: i == 0, // First account is signer
                is_writable: i % 2 == 0, // Even indices are writable
            })
            .collect()
    }

    /// System program ID constant
    const SYSTEM_PROGRAM_ID: &'static str = "11111111111111111111111111111111";

    /// Generate a simple transfer instruction for testing
    pub fn simple_transfer_instruction(
        from: Pubkey,
        to: Pubkey,
        amount: u64,
    ) -> Instruction {
        Instruction {
            program_id: Pubkey::from_base58(Self::SYSTEM_PROGRAM_ID).unwrap(), // System program
            accounts: vec![
                AccountMeta {
                    pubkey: from,
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: to,
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: {
                let mut data = vec![2, 0, 0, 0]; // Transfer instruction
                data.extend_from_slice(&amount.to_le_bytes());
                data
            },
        }
    }
}

#[cfg(test)]
#[cfg(feature = "testing")]
mod tests {
    use super::*;

    #[test]
    fn test_random_keypair() {
        let keypair1 = TestDataGenerator::random_keypair();
        let keypair2 = TestDataGenerator::random_keypair();

        // Different keypairs should have different public keys
        assert_ne!(keypair1.pubkey(), keypair2.pubkey());
    }

    #[test]
    fn test_random_pubkey() {
        let pubkey1 = TestDataGenerator::random_pubkey();
        let pubkey2 = TestDataGenerator::random_pubkey();

        // Different calls should produce different pubkeys
        assert_ne!(pubkey1, pubkey2);
    }

    #[test]
    fn test_sample_account_data() {
        let data1 = TestDataGenerator::sample_account_data(32);
        let data2 = TestDataGenerator::sample_account_data(32);

        assert_eq!(data1.len(), 32);
        assert_eq!(data2.len(), 32);
        // Random data should be different
        assert_ne!(data1, data2);
    }

    #[test]
    fn test_sample_instruction() {
        let instruction = TestDataGenerator::sample_instruction();

        assert_eq!(instruction.accounts.len(), 2);
        assert!(!instruction.data.is_empty());
        assert!(instruction.accounts[0].is_signer);
        assert!(instruction.accounts[0].is_writable);
        assert!(!instruction.accounts[1].is_signer);
        assert!(!instruction.accounts[1].is_writable);
    }

    #[test]
    fn test_sample_transaction() {
        let transaction = TestDataGenerator::sample_transaction();

        assert_eq!(transaction.signatures.len(), 1);
        assert!(!transaction.message.account_keys.is_empty());
        assert!(!transaction.message.instructions.is_empty());
    }

    #[test]
    fn test_deterministic_generation() {
        let keypair1 = TestDataGenerator::keypair_from_seed(42);
        let keypair2 = TestDataGenerator::keypair_from_seed(42);

        // Same seed should produce same keypair
        assert_eq!(keypair1.pubkey(), keypair2.pubkey());

        let pubkey1 = TestDataGenerator::pubkey_from_seed(42);
        let pubkey2 = TestDataGenerator::pubkey_from_seed(42);

        // Same seed should produce same pubkey
        assert_eq!(pubkey1, pubkey2);
    }

    #[test]
    fn test_test_keypairs() {
        let keypairs = TestDataGenerator::test_keypairs(5);
        assert_eq!(keypairs.len(), 5);

        // All keypairs should be different
        for i in 0..keypairs.len() {
            for j in (i + 1)..keypairs.len() {
                assert_ne!(keypairs[i].pubkey(), keypairs[j].pubkey());
            }
        }
    }

    #[test]
    fn test_simple_transfer_instruction() {
        let from = TestDataGenerator::random_pubkey();
        let to = TestDataGenerator::random_pubkey();
        let amount = 1_000_000_000; // 1 SOL

        let instruction = TestDataGenerator::simple_transfer_instruction(from, to, amount);

        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, from);
        assert_eq!(instruction.accounts[1].pubkey, to);
        assert!(instruction.accounts[0].is_signer);
        assert!(!instruction.accounts[1].is_signer);
        assert!(instruction.accounts[0].is_writable);
        assert!(instruction.accounts[1].is_writable);
    }
}