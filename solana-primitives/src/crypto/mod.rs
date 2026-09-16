use crate::error::{Result, SolanaError};
use crate::types::{Pubkey, SignatureBytes, VersionedTransaction};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

fn signing_key(private_key: &[u8]) -> Result<SigningKey> {
    let seed = <[u8; 32]>::try_from(private_key).map_err(|_| SolanaError::InvalidLength {
        expected: 32,
        actual: private_key.len(),
    })?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Get the public key for a 32-byte Ed25519 private key (seed).
pub fn get_public_key(private_key: &[u8]) -> Result<[u8; 32]> {
    Ok(signing_key(private_key)?.verifying_key().to_bytes())
}

/// Get a Solana address (base58 encoded public key) from a private key
pub fn get_address(private_key: &[u8]) -> Result<String> {
    get_public_key(private_key).map(|key| Pubkey::new(key).to_base58())
}

/// Get a Solana address from a public key
pub fn get_address_from_public_key(public_key: &[u8]) -> Result<String> {
    Pubkey::try_from(public_key).map(|key| key.to_base58())
}

/// Sign a message with a 32-byte Ed25519 private key (seed).
pub fn sign_message(private_key: &[u8], message: &[u8]) -> Result<SignatureBytes> {
    Ok(SignatureBytes::new(
        signing_key(private_key)?.sign(message).to_bytes(),
    ))
}

/// Verify an Ed25519 signature over `message`.
pub fn verify_signature(pubkey: &Pubkey, message: &[u8], signature: &SignatureBytes) -> Result<()> {
    let verifying_key =
        VerifyingKey::from_bytes(pubkey.as_bytes()).map_err(|_| SolanaError::InvalidPublicKey)?;
    verifying_key
        .verify(
            message,
            &ed25519_dalek::Signature::from_bytes(signature.as_bytes()),
        )
        .map_err(|_| SolanaError::InvalidSignature)
}

/// Verify that a transaction is sanitized and every required signature is valid.
///
/// Legacy [`Transaction`](crate::Transaction)s can use [`Transaction::verify`](crate::Transaction::verify).
pub fn verify_transaction(transaction: &VersionedTransaction) -> Result<()> {
    transaction.verify()
}

/// Hash data using SHA-256
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::key;

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
