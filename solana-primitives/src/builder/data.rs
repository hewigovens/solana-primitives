//! A low-level byte writer for instruction data.

use crate::types::Pubkey;

/// Appends fields to an instruction data buffer.
///
/// Integers are little-endian. Programs use different ABIs (Borsh, bincode,
/// hand-packed layouts), so each helper documents its exact encoding; pick the
/// one that matches the target program.
#[derive(Debug, Clone, Default)]
pub struct InstructionDataBuilder {
    data: Vec<u8>,
}

impl InstructionDataBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a one-byte discriminant. Use [`Self::u32`] for bincode enums and
    /// [`Self::bytes`] for 8-byte Anchor discriminators.
    pub fn instruction(self, discriminant: u8) -> Self {
        self.u8(discriminant)
    }

    /// Append raw bytes.
    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.data.extend_from_slice(bytes);
        self
    }

    pub fn u8(self, value: u8) -> Self {
        self.bytes(&[value])
    }

    pub fn u16(self, value: u16) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn u32(self, value: u32) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn u64(self, value: u64) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn u128(self, value: u128) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn i8(self, value: i8) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn i16(self, value: i16) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn i32(self, value: i32) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn i64(self, value: i64) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    pub fn i128(self, value: i128) -> Self {
        self.bytes(&value.to_le_bytes())
    }

    /// Append a bool as one byte (0 or 1).
    pub fn bool(self, value: bool) -> Self {
        self.u8(value.into())
    }

    /// Append the 32 key bytes.
    pub fn pubkey(self, pubkey: &Pubkey) -> Self {
        self.bytes(pubkey.as_bytes())
    }

    /// Append an optional pubkey as a one-byte tag (0 = none, 1 = some) followed
    /// by the key when present, as Borsh `Option<Pubkey>` and SPL Token
    /// instruction data encode it.
    pub fn option_pubkey(self, pubkey: Option<&Pubkey>) -> Self {
        match pubkey {
            Some(pubkey) => self.u8(1).pubkey(pubkey),
            None => self.u8(0),
        }
    }

    /// Append a string as a `u32` byte length and its UTF-8 bytes (Borsh `String`).
    ///
    /// # Panics
    ///
    /// Panics if the string is longer than `u32::MAX` bytes.
    pub fn string_u32_le(self, s: &str) -> Self {
        let len = u32::try_from(s.len()).expect("string length exceeds u32::MAX");
        self.u32(len).bytes(s.as_bytes())
    }

    /// Append a string as a `u64` byte length and its UTF-8 bytes (bincode `String`,
    /// used by the System program).
    pub fn string_u64_le(self, s: &str) -> Self {
        self.u64(s.len() as u64).bytes(s.as_bytes())
    }

    /// Append a Borsh string; same as [`Self::string_u32_le`].
    #[deprecated(since = "0.3.0", note = "use `string_u32_le` or `string_u64_le`")]
    pub fn string(self, s: &str) -> Self {
        self.string_u32_le(s)
    }

    /// Return the encoded bytes.
    pub fn build(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::system::SystemInstruction;
    use hexlit::hex;

    #[test]
    fn integers_are_little_endian() {
        let data = InstructionDataBuilder::new()
            .instruction(3)
            .u16(0x0102)
            .u32(0x0304_0506)
            .u64(1_000_000)
            .u128(1)
            .i8(-1)
            .i16(-2)
            .i32(-3)
            .i64(-4)
            .i128(-5)
            .bool(true)
            .build();
        assert_eq!(
            data,
            hex!(
                "03 0201 06050403 40420f0000000000 01000000000000000000000000000000"
                "ff feff fdffffff fcffffffffffffff fbffffffffffffffffffffffffffffff"
                "01"
            )
        );
    }

    #[test]
    fn pubkeys_and_options() {
        let key = Pubkey::new([2; 32]);
        let data = InstructionDataBuilder::new()
            .pubkey(&key)
            .option_pubkey(Some(&key))
            .option_pubkey(None)
            .build();
        assert_eq!(data, [&[2; 32][..], &[1], &[2; 32], &[0]].concat());
    }

    #[test]
    fn string_length_prefixes() {
        let data = InstructionDataBuilder::new()
            .string_u32_le("hello")
            .string_u64_le("hi")
            .build();
        assert_eq!(data, hex!("05000000 68656c6c6f 0200000000000000 6869"));
    }

    #[test]
    fn matches_system_program_encoding() {
        let (base, owner) = (Pubkey::new([1; 32]), Pubkey::new([3; 32]));
        let data = InstructionDataBuilder::new()
            .u32(10)
            .pubkey(&base)
            .string_u64_le("seed")
            .pubkey(&owner)
            .build();
        let expected = SystemInstruction::AssignWithSeed {
            base,
            seed: "seed".to_string(),
            owner,
        }
        .serialize();
        assert_eq!(data, expected);
    }
}
