//! Signature operations and verification

use crate::{
    types::Pubkey,
    error::{Result, SolanaError},
};
use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};

/// A cryptographic signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    bytes: [u8; 64],
}

impl Signature {
    /// Create a new signature from bytes
    pub fn new(bytes: [u8; 64]) -> Self {
        Self { bytes }
    }

    /// Get the signature bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.bytes
    }

    /// Convert to base58 string
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.bytes).into_string()
    }

    /// Create from base58 string
    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| SolanaError::Crypto(format!("Invalid base58 signature: {}", e)))?;
        
        if bytes.len() != 64 {
            return Err(SolanaError::Crypto("Signature must be 64 bytes".to_string()));
        }

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&bytes);
        Ok(Self::new(sig_bytes))
    }
}

/// Signature verification utilities
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verify a signature against a message and public key
    pub fn verify(
        signature: &Signature,
        message: &[u8],
        public_key: &Pubkey,
    ) -> Result<bool> {
        let ed25519_signature = match Ed25519Signature::try_from(signature.as_bytes().as_slice()) {
            Ok(sig) => sig,
            Err(e) => return Err(SolanaError::Crypto(format!("Invalid signature format: {}", e))),
        };
        
        let verifying_key = VerifyingKey::from_bytes(public_key.as_bytes())
            .map_err(|e| SolanaError::Crypto(format!("Invalid public key format: {}", e)))?;

        match verifying_key.verify(message, &ed25519_signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Verify multiple signatures in batch (more efficient for many signatures)
    pub fn verify_batch(
        signatures: &[Signature],
        messages: &[&[u8]],
        public_keys: &[Pubkey],
    ) -> Result<Vec<bool>> {
        if signatures.len() != messages.len() || messages.len() != public_keys.len() {
            return Err(SolanaError::Crypto(
                "Signatures, messages, and public keys must have the same length".to_string()
            ));
        }

        let mut results = Vec::with_capacity(signatures.len());
        
        for ((signature, message), public_key) in signatures.iter()
            .zip(messages.iter())
            .zip(public_keys.iter()) {
            let result = Self::verify(signature, message, public_key)?;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Keypair;

    #[test]
    fn test_signature_creation() {
        let bytes = [1u8; 64];
        let signature = Signature::new(bytes);
        assert_eq!(signature.as_bytes(), &bytes);
    }

    #[test]
    fn test_signature_base58_roundtrip() {
        let bytes = [42u8; 64];
        let signature = Signature::new(bytes);
        let base58_str = signature.to_base58();
        let recovered_signature = Signature::from_base58(&base58_str).unwrap();
        
        assert_eq!(signature, recovered_signature);
    }

    #[test]
    fn test_signature_verification() {
        let keypair = Keypair::generate();
        let message = b"test message";
        let signature = keypair.sign_message(message);
        
        let is_valid = SignatureVerifier::verify(&signature, message, keypair.pubkey()).unwrap();
        assert!(is_valid);
        
        // Test with wrong message
        let wrong_message = b"wrong message";
        let is_valid = SignatureVerifier::verify(&signature, wrong_message, keypair.pubkey()).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_batch_verification() {
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        
        let message1 = b"message 1";
        let message2 = b"message 2";
        
        let signature1 = keypair1.sign_message(message1);
        let signature2 = keypair2.sign_message(message2);
        
        let signatures = vec![signature1, signature2];
        let messages = vec![message1.as_slice(), message2.as_slice()];
        let public_keys = vec![*keypair1.pubkey(), *keypair2.pubkey()];
        
        let results = SignatureVerifier::verify_batch(&signatures, &messages, &public_keys).unwrap();
        assert_eq!(results, vec![true, true]);
    }

    #[test]
    fn test_batch_verification_length_mismatch() {
        let signatures = vec![Signature::new([0u8; 64])];
        let messages = vec![b"message".as_slice()];
        let public_keys = vec![]; // Empty, causing length mismatch
        
        let result = SignatureVerifier::verify_batch(&signatures, &messages, &public_keys);
        assert!(result.is_err());
    }
}