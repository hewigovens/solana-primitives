use crate::error::{Result, SolanaError};
use borsh::{BorshDeserialize, BorshSerialize};

/// Helper function to convert a compact array to bytes
pub fn compact_array_to_bytes<T: BorshSerialize>(items: &[T]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    items
        .serialize(&mut &mut bytes)
        .map_err(|e| SolanaError::Serialization(e.to_string()))?;
    Ok(bytes)
}

/// Helper function to convert bytes to a compact array
pub fn bytes_to_compact_array<T: BorshDeserialize>(bytes: &[u8]) -> Result<Vec<T>> {
    let mut bytes_mut = bytes; // Borsh deserialize expects a mutable slice for some reason
    Vec::<T>::deserialize(&mut bytes_mut).map_err(|e| SolanaError::Serialization(e.to_string()))
}
