//! Instruction data builder for encoding instruction parameters

use crate::types::Pubkey;

/// Builder for encoding instruction data
pub struct InstructionDataBuilder {
    data: Vec<u8>,
}

impl InstructionDataBuilder {
    /// Create a new instruction data builder
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Add a discriminant/instruction type byte
    pub fn instruction(mut self, discriminant: u8) -> Self {
        self.data.push(discriminant);
        self
    }

    /// Add raw bytes
    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.data.extend_from_slice(bytes);
        self
    }

    /// Add a u8 value
    pub fn u8(mut self, value: u8) -> Self {
        self.data.push(value);
        self
    }

    /// Add a u16 value (little-endian)
    pub fn u16(mut self, value: u16) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a u32 value (little-endian)
    pub fn u32(mut self, value: u32) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a u64 value (little-endian)
    pub fn u64(mut self, value: u64) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a u128 value (little-endian)
    pub fn u128(mut self, value: u128) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a i8 value
    pub fn i8(mut self, value: i8) -> Self {
        self.data.push(value as u8);
        self
    }

    /// Add a i16 value (little-endian)
    pub fn i16(mut self, value: i16) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a i32 value (little-endian)
    pub fn i32(mut self, value: i32) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a i64 value (little-endian)
    pub fn i64(mut self, value: i64) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a i128 value (little-endian)
    pub fn i128(mut self, value: i128) -> Self {
        self.data.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Add a bool value
    pub fn bool(mut self, value: bool) -> Self {
        self.data.push(value as u8);
        self
    }

    /// Add a pubkey (32 bytes)
    pub fn pubkey(mut self, pubkey: &Pubkey) -> Self {
        self.data.extend_from_slice(pubkey.as_bytes());
        self
    }

    /// Add an optional pubkey
    pub fn option_pubkey(mut self, pubkey: Option<&Pubkey>) -> Self {
        match pubkey {
            Some(pk) => {
                self.data.push(1);
                self.data.extend_from_slice(pk.as_bytes());
            }
            None => {
                self.data.push(0);
            }
        }
        self
    }

    /// Add a string (with length prefix as u32)
    pub fn string(mut self, s: &str) -> Self {
        let bytes = s.as_bytes();
        self.data
            .extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.data.extend_from_slice(bytes);
        self
    }

    /// Build the final data vector
    pub fn build(self) -> Vec<u8> {
        self.data
    }
}

impl Default for InstructionDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Pubkey;

    #[test]
    fn test_instruction_data_builder() {
        let data = InstructionDataBuilder::new()
            .instruction(3)
            .u64(1000000)
            .u8(6)
            .bool(true)
            .build();

        let expected = vec![
            3, // instruction
            0x40, 0x42, 0x0F, 0, 0, 0, 0, 0, // u64(1000000)
            6, // u8(6)
            1, // bool(true)
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_instruction_data_builder_with_pubkey() {
        let pubkey = Pubkey::new([1u8; 32]);

        let data = InstructionDataBuilder::new()
            .instruction(0)
            .pubkey(&pubkey)
            .build();

        let mut expected = vec![0];
        expected.extend_from_slice(&[1u8; 32]);

        assert_eq!(data, expected);
    }

    #[test]
    fn test_instruction_data_builder_with_option() {
        let pubkey = Pubkey::new([2u8; 32]);

        // Test with Some value
        let data = InstructionDataBuilder::new()
            .instruction(1)
            .option_pubkey(Some(&pubkey))
            .build();

        let mut expected = vec![1, 1]; // instruction, Some flag
        expected.extend_from_slice(&[2u8; 32]);
        assert_eq!(data, expected);

        // Test with None value
        let data_none = InstructionDataBuilder::new()
            .instruction(1)
            .option_pubkey(None)
            .build();

        assert_eq!(data_none, vec![1, 0]); // instruction, None flag
    }

    #[test]
    fn test_instruction_data_builder_with_string() {
        let data = InstructionDataBuilder::new()
            .instruction(5)
            .string("hello")
            .build();

        let expected = vec![
            5, // instruction
            5, 0, 0, 0, // string length (u32)
            b'h', b'e', b'l', b'l', b'o', // string bytes
        ];

        assert_eq!(data, expected);
    }
}
