use crate::error::{Result, SolanaError};
use crate::types::{Pubkey, SignatureBytes, Transaction};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// Get the public key from a private key
pub fn get_public_key(private_key: &[u8]) -> Result<[u8; 32]> {
    if private_key.len() != 32 {
        return Err(SolanaError::InvalidSignature(format!(
            "invalid private key length: {}, expected: 32",
            private_key.len()
        )));
    }

    let signing_key = SigningKey::try_from(private_key)
        .map_err(|_| SolanaError::InvalidSignature("failed to create signing key".to_string()))?;

    Ok(signing_key.verifying_key().to_bytes())
}

/// Get a Solana address (base58 encoded public key) from a private key
pub fn get_address(private_key: &[u8]) -> Result<String> {
    let public_key = get_public_key(private_key)?;
    let pubkey = Pubkey::new(public_key);
    Ok(pubkey.to_base58())
}

/// Get a Solana address from a public key
pub fn get_address_from_public_key(public_key: &[u8]) -> Result<String> {
    if public_key.len() != 32 {
        return Err(SolanaError::InvalidPubkey(format!(
            "invalid public key length: {}, expected: 32",
            public_key.len()
        )));
    }

    let mut pk_bytes = [0u8; 32];
    pk_bytes.copy_from_slice(public_key);
    let pubkey = Pubkey::new(pk_bytes);

    Ok(pubkey.to_base58())
}

/// Verify that a transaction's signatures are valid
pub fn verify_transaction(transaction: &Transaction) -> Result<()> {
    let required = transaction.message.header.num_required_signatures as usize;
    if transaction.signatures.len() != required {
        return Err(SolanaError::InvalidSignature(format!(
            "signature count mismatch: found {}, required {}",
            transaction.signatures.len(),
            required
        )));
    }

    // Get the message bytes that were signed
    let message_bytes = transaction
        .message
        .serialize_for_signing()
        .map_err(SolanaError::SerializationError)?;

    for (i, signature) in transaction.signatures.iter().enumerate() {
        let signer_pubkey = &transaction.message.account_keys[i];
        let verifying_key = VerifyingKey::from_bytes(signer_pubkey.as_bytes()).map_err(|_| {
            SolanaError::InvalidPubkey("failed to create verifying key from pubkey".to_string())
        })?;

        // Convert our SignatureBytes to the ed25519_dalek Signature type
        let sig_bytes = signature.as_bytes();
        if sig_bytes.len() != 64 {
            return Err(SolanaError::InvalidSignature(format!(
                "invalid signature length: {}, expected: 64",
                sig_bytes.len()
            )));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(sig_bytes);

        let dalek_signature = ed25519_dalek::Signature::from_bytes(&sig_array);

        verifying_key
            .verify(&message_bytes, &dalek_signature)
            .map_err(|_| {
                SolanaError::InvalidSignature("signature verification failed".to_string())
            })?;
    }

    Ok(())
}

/// Sign a message with a private key
pub fn sign_message(private_key: &[u8], message: &[u8]) -> Result<SignatureBytes> {
    if private_key.len() != 32 {
        return Err(SolanaError::InvalidSignature(format!(
            "invalid private key length: {}, expected: 32",
            private_key.len()
        )));
    }

    let signing_key = SigningKey::try_from(private_key)
        .map_err(|_| SolanaError::InvalidSignature("failed to create signing key".to_string()))?;

    let signature = signing_key.sign(message);
    Ok(SignatureBytes::new(signature.to_bytes()))
}

/// Hash data using SHA-256
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Message, MessageHeader};

    fn build_message(signer: Pubkey) -> Message {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 0,
        };
        Message::new(header, vec![signer], [0u8; 32], Vec::new())
    }

    #[test]
    fn verify_transaction_rejects_missing_signatures() {
        let private_key = [1u8; 32];
        let public_key = get_public_key(&private_key).expect("valid key");
        let signer = Pubkey::new(public_key);

        let transaction = Transaction::new(build_message(signer));

        let result = verify_transaction(&transaction);
        assert!(result.is_err());
    }

    #[test]
    fn verify_transaction_accepts_properly_signed_transaction() {
        let private_key = [1u8; 32];
        let public_key = get_public_key(&private_key).expect("valid key");
        let signer = Pubkey::new(public_key);

        let mut transaction = Transaction::new(build_message(signer));
        transaction.sign(&[&private_key]).expect("sign succeeds");

        let result = verify_transaction(&transaction);
        assert!(result.is_ok());
    }
}
