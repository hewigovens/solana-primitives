pub use crate::error::{Result, SolanaError};
pub use borsh::{BorshDeserialize, BorshSerialize};
pub mod short_vec;

/// Helper function to convert a compact array to bytes
pub fn compact_array_to_bytes<T: BorshSerialize>(items: &[T]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    items
        .serialize(&mut &mut bytes)
        .map_err(|e| SolanaError::SerializationError(e.to_string()))?;
    Ok(bytes)
}

/// Helper function to convert bytes to a compact array
pub fn bytes_to_compact_array<T: BorshDeserialize>(bytes: &[u8]) -> Result<Vec<T>> {
    let mut bytes = bytes;
    Vec::<T>::deserialize(&mut bytes).map_err(|e| SolanaError::SerializationError(e.to_string()))
}

// Re-export the ShortVec and short_vec serialization
pub use short_vec::{ShortU16, ShortVec};
