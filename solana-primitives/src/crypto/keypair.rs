//! Keypair generation and management

use crate::{
    types::Pubkey,
    error::{Result, SolanaError},
};
use ed25519_dalek::{SigningKey, Signer};
use rand::{rngs::OsRng, RngCore};

/// A cryptographic keypair for Solana
#[derive(Debug, Clone)]
pub struct Keypair {
    signing_key: SigningKey,
    public_key: Pubkey,
}

impl Keypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();
        let public_key = Pubkey::new(verifying_key.to_bytes());
        
        Self {
            signing_key,
            public_key,
        }
    }

    /// Create a keypair from a seed
    pub fn from_seed(seed: &[u8]) -> Result<Self> {
        if seed.len() != 32 {
            return Err(SolanaError::Crypto("Seed must be exactly 32 bytes".to_string()));
        }

        let mut seed_array = [0u8; 32];
        seed_array.copy_from_slice(seed);
        
        let signing_key = SigningKey::from_bytes(&seed_array);
        let verifying_key = signing_key.verifying_key();
        let public_key = Pubkey::new(verifying_key.to_bytes());

        Ok(Self {
            signing_key,
            public_key,
        })
    }

    /// Create a keypair from a base58 string
    pub fn from_base58_string(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| SolanaError::Crypto(format!("Invalid base58: {e}")))?;
        
        Self::from_seed(&bytes)
    }

    /// Get the public key
    pub fn pubkey(&self) -> &Pubkey {
        &self.public_key
    }

    /// Sign a message
    pub fn sign_message(&self, message: &[u8]) -> crate::crypto::Signature {
        let signature = self.signing_key.sign(message);
        crate::crypto::Signature::new(signature.to_bytes())
    }

    /// Get the secret key bytes
    pub fn secret_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Convert to base58 string representation
    pub fn to_base58_string(&self) -> String {
        bs58::encode(self.secret_key_bytes()).into_string()
    }
}

impl PartialEq for Keypair {
    fn eq(&self, other: &Self) -> bool {
        // Compare both public and private keys for true equality
        self.public_key == other.public_key && self.signing_key == other.signing_key
    }
}

impl Eq for Keypair {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        
        // Different keypairs should have different public keys
        assert_ne!(keypair1.pubkey(), keypair2.pubkey());
    }

    #[test]
    fn test_keypair_from_seed() {
        let seed = [1u8; 32];
        let keypair1 = Keypair::from_seed(&seed).unwrap();
        let keypair2 = Keypair::from_seed(&seed).unwrap();
        
        // Same seed should produce same keypair
        assert_eq!(keypair1, keypair2);
    }

    #[test]
    fn test_keypair_from_invalid_seed() {
        let seed = [1u8; 16]; // Wrong length
        let result = Keypair::from_seed(&seed);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_message() {
        let keypair = Keypair::generate();
        let message = b"test message";
        
        let signature = keypair.sign_message(message);
        assert_eq!(signature.as_bytes().len(), 64);
    }

    #[test]
    fn test_base58_roundtrip() {
        let keypair = Keypair::generate();
        let base58_str = keypair.to_base58_string();
        let recovered_keypair = Keypair::from_base58_string(&base58_str).unwrap();
        
        assert_eq!(keypair, recovered_keypair);
    }
}