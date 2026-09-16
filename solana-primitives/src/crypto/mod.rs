//! Hashing, and (with the `signing` feature) Ed25519 keys and signatures.

use crate::error::Result;
use crate::types::Pubkey;
use sha2::{Digest, Sha256};
#[cfg(feature = "signing")]
use {
    crate::error::SolanaError,
    crate::types::{SignatureBytes, VersionedTransaction},
    ed25519_dalek::{Signer, SigningKey, VerifyingKey},
};

#[cfg(feature = "signing")]
fn signing_key(private_key: &[u8]) -> Result<SigningKey> {
    let seed = <[u8; 32]>::try_from(private_key).map_err(|_| SolanaError::InvalidLength {
        expected: 32,
        actual: private_key.len(),
    })?;
    Ok(SigningKey::from_bytes(&seed))
}

#[cfg(feature = "signing")]
/// Get the public key for a 32-byte Ed25519 private key (seed).
pub fn get_public_key(private_key: &[u8]) -> Result<[u8; 32]> {
    Ok(signing_key(private_key)?.verifying_key().to_bytes())
}

#[cfg(feature = "signing")]
/// Get a Solana address (base58 encoded public key) from a private key
pub fn get_address(private_key: &[u8]) -> Result<String> {
    get_public_key(private_key).map(|key| Pubkey::new(key).to_base58())
}

/// Get a Solana address from a public key
pub fn get_address_from_public_key(public_key: &[u8]) -> Result<String> {
    Pubkey::try_from(public_key).map(|key| key.to_base58())
}

#[cfg(feature = "signing")]
/// Sign a message with a 32-byte Ed25519 private key (seed).
pub fn sign_message(private_key: &[u8], message: &[u8]) -> Result<SignatureBytes> {
    Ok(SignatureBytes::new(
        signing_key(private_key)?.sign(message).to_bytes(),
    ))
}

#[cfg(feature = "signing")]
/// Verify an Ed25519 signature over `message` with the checks Solana applies:
/// small-order public keys and `R` points, and non-canonical `s`, are rejected.
pub fn verify_signature(pubkey: &Pubkey, message: &[u8], signature: &SignatureBytes) -> Result<()> {
    let verifying_key =
        VerifyingKey::from_bytes(pubkey.as_bytes()).map_err(|_| SolanaError::InvalidPublicKey)?;
    verifying_key
        .verify_strict(
            message,
            &ed25519_dalek::Signature::from_bytes(signature.as_bytes()),
        )
        .map_err(|_| SolanaError::InvalidSignature)
}

#[cfg(feature = "signing")]
/// Verify that a transaction is sanitized and every required signature is valid.
pub fn verify_transaction(transaction: &VersionedTransaction) -> Result<()> {
    transaction.verify()
}

/// Hash data using SHA-256
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

#[cfg(all(test, feature = "signing"))]
mod tests {
    use super::*;
    use crate::test_utils::key;
    use hexlit::hex;

    #[test]
    fn key_derivation_and_signatures() {
        let private_key = crate::crypto::hash_data(b"payer");
        let public_key = get_public_key(&private_key).unwrap();
        assert_eq!(
            Pubkey::new(public_key).to_base58(),
            "ECbPvoRPTunYYuu6iCP8gK4GzGX4nc5rPsUpAKoT6vV4"
        );
        assert_eq!(
            get_address(&private_key).unwrap(),
            get_address_from_public_key(&public_key).unwrap()
        );

        let signature = sign_message(&private_key, b"hello").unwrap();
        let pubkey = Pubkey::new(public_key);
        assert_eq!(verify_signature(&pubkey, b"hello", &signature), Ok(()));
        assert_eq!(
            verify_signature(&pubkey, b"hullo", &signature),
            Err(SolanaError::InvalidSignature)
        );
        assert_eq!(
            verify_signature(&key("other"), b"hello", &signature),
            Err(SolanaError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_small_order_keys() {
        // The identity point as the public key with R = basepoint, s = 1 satisfies the
        // unbatched equation for every message, but Solana rejects small-order keys.
        let identity = Pubkey::new(hex!(
            "0100000000000000000000000000000000000000000000000000000000000000"
        ));
        let signature = SignatureBytes::new(hex!(
            "5866666666666666666666666666666666666666666666666666666666666666"
            "0100000000000000000000000000000000000000000000000000000000000000"
        ));
        for message in [&b"any"[..], b"message"] {
            assert_eq!(
                verify_signature(&identity, message, &signature),
                Err(SolanaError::InvalidSignature)
            );
        }
    }

    #[test]
    fn rejects_non_canonical_s() {
        let private_key = crate::crypto::hash_data(b"payer");
        let pubkey = Pubkey::new(get_public_key(&private_key).unwrap());
        let signature = sign_message(&private_key, b"hello").unwrap();
        // s + l encodes the same scalar, but only the reduced form is accepted.
        const L: [u8; 32] =
            hex!("edd3f55c1a631258d69cf7a2def9de1400000000000000000000000000000010");
        let mut bytes = *signature.as_bytes();
        let mut carry = 0u16;
        for (byte, l) in bytes[32..].iter_mut().zip(L) {
            let sum = u16::from(*byte) + u16::from(l) + carry;
            *byte = sum as u8;
            carry = sum >> 8;
        }
        assert_eq!(carry, 0);
        assert_eq!(
            verify_signature(&pubkey, b"hello", &SignatureBytes::new(bytes)),
            Err(SolanaError::InvalidSignature)
        );
    }

    #[test]
    fn rejects_bad_key_lengths() {
        let short = [1u8; 31];
        let err = SolanaError::InvalidLength {
            expected: 32,
            actual: 31,
        };
        assert_eq!(get_public_key(&short), Err(err.clone()));
        assert_eq!(sign_message(&short, b"m"), Err(err.clone()));
        assert_eq!(get_address_from_public_key(&short), Err(err));
    }
}
