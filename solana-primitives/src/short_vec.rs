// Compact serde-encoding of vectors with small length.
use borsh::{BorshDeserialize, BorshSerialize};

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{self, SerializeTuple, Serializer},
};
use std::{convert::TryFrom, fmt, marker::PhantomData, vec::Vec};

/// Represents a ShortU16.
///
/// Same as u16, but serialized with 1 to 3 bytes. If the value is above
/// 0x7f, the top bit is set and the remaining value is stored in the next
/// bytes. Each byte follows the same pattern until the 3rd byte. The 3rd
/// byte, if needed, uses all 8 bits to store the last byte of the original
/// value.
pub struct ShortU16(pub u16);

impl Serialize for ShortU16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Pass a non-zero value to serialize_tuple() so that serde_json will
        // generate an open bracket.
        let mut seq = serializer.serialize_tuple(1)?;
        let mut rem_val = self.0;
        loop {
            let mut elem = (rem_val & 0x7f) as u8;
            rem_val >>= 7;
            if rem_val == 0 {
                seq.serialize_element(&elem)?;
                break;
            } else {
                elem |= 0x80;
                seq.serialize_element(&elem)?;
            }
        }
        seq.end()
    }
}

enum VisitStatus {
    Done(u16),
    More(u16),
}

#[derive(Debug)]
enum VisitError {
    TooLong(usize),
    TooShort(usize),
    Overflow(u32),
    Alias,
    ByteThreeContinues,
}

impl VisitError {
    fn into_de_error<'de, A>(self) -> A::Error
    where
        A: SeqAccess<'de>,
    {
        match self {
            VisitError::TooLong(len) => de::Error::invalid_length(len, &"three or fewer bytes"),
            VisitError::TooShort(len) => de::Error::invalid_length(len, &"more bytes"),
            VisitError::Overflow(val) => de::Error::invalid_value(
                de::Unexpected::Unsigned(val as u64),
                &"a value in the range [0, 65535]",
            ),
            VisitError::Alias => de::Error::invalid_value(
                de::Unexpected::Other("alias encoding"),
                &"strict form encoding",
            ),
            VisitError::ByteThreeContinues => de::Error::invalid_value(
                de::Unexpected::Other("continue signal on byte-three"),
                &"a terminal signal on or before byte-three",
            ),
        }
    }
}

type VisitResult = Result<VisitStatus, VisitError>;

const MAX_ENCODING_LENGTH: usize = 3;

fn visit_byte(elem: u8, val: u16, nth_byte: usize) -> VisitResult {
    if elem == 0 && nth_byte != 0 {
        return Err(VisitError::Alias);
    }
    let val = u32::from(val);
    let elem = u32::from(elem);
    let elem_val = elem & 0x7f;
    let elem_done = (elem & 0x80) == 0;
    if nth_byte >= MAX_ENCODING_LENGTH {
        return Err(VisitError::TooLong(nth_byte.saturating_add(1)));
    } else if nth_byte == MAX_ENCODING_LENGTH.saturating_sub(1) && !elem_done {
        return Err(VisitError::ByteThreeContinues);
    }
    let shift = u32::try_from(nth_byte)
        .unwrap_or(u32::MAX)
        .saturating_mul(7);
    let elem_val = elem_val.checked_shl(shift).unwrap_or(u32::MAX);
    let new_val = val | elem_val;
    let val = u16::try_from(new_val).map_err(|_| VisitError::Overflow(new_val))?;
    if elem_done {
        Ok(VisitStatus::Done(val))
    } else {
        Ok(VisitStatus::More(val))
    }
}

struct ShortU16Visitor;

impl<'de> Visitor<'de> for ShortU16Visitor {
    type Value = ShortU16;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a ShortU16")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<ShortU16, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Decodes an unsigned 16 bit integer one-to-one encoded as follows:
        // 1 byte : 0xxxxxxx => 00000000 0xxxxxxx : 0 - 127
        // 2 bytes : 1xxxxxxx 0yyyyyyy => 00yyyyyy yxxxxxxx : 128 - 16,383
        // 3 bytes : 1xxxxxxx 1yyyyyyy 000000zz => zzyyyyyy yxxxxxxx : 16,384 - 65,535
        let mut val: u16 = 0;
        for nth_byte in 0..MAX_ENCODING_LENGTH {
            let elem: u8 = seq.next_element()?.ok_or_else(|| {
                VisitError::TooShort(nth_byte.saturating_add(1)).into_de_error::<A>()
            })?;
            match visit_byte(elem, val, nth_byte).map_err(|e| e.into_de_error::<A>())? {
                VisitStatus::Done(new_val) => return Ok(ShortU16(new_val)),
                VisitStatus::More(new_val) => val = new_val,
            }
        }
        Err(VisitError::ByteThreeContinues.into_de_error::<A>())
    }
}

impl<'de> Deserialize<'de> for ShortU16 {
    fn deserialize<D>(deserializer: D) -> Result<ShortU16, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(3, ShortU16Visitor)
    }
}

/// If you don't want to use the ShortVec newtype, you can do ShortVec
/// serialization on an ordinary vector with the following field annotation:
///
/// #[serde(with = "short_vec")]
pub fn serialize<S: Serializer, T: Serialize>(
    elements: &[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // Pass a non-zero value to serialize_tuple() so that serde_json will
    // generate an open bracket.
    let mut seq = serializer.serialize_tuple(1)?;
    let len = elements.len();
    if len > u16::MAX as usize {
        return Err(ser::Error::custom("length larger than u16"));
    }
    let short_len = ShortU16(len as u16);
    seq.serialize_element(&short_len)?;
    for element in elements {
        seq.serialize_element(element)?;
    }
    seq.end()
}

struct ShortVecVisitor<T> {
    _t: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for ShortVecVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Vec with a multi-byte length")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let short_len: ShortU16 = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let len = short_len.0 as usize;
        // Cap the reservation so a malformed payload claiming a large count can't allocate big.
        let mut result = Vec::with_capacity(len.min(1024));
        for i in 0..len {
            let elem = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(i, &self))?;
            result.push(elem);
        }
        Ok(result)
    }
}

// Helper function to encode a usize length into Compact-U16 format bytes.
// Returns a Vec<u8> with the encoded length or an Err if length is too large for u16.
pub fn encode_length_to_compact_u16_bytes(len: usize) -> Result<Vec<u8>, String> {
    if len > u16::MAX as usize {
        return Err(format!(
            "Length {len} exceeds u16::MAX, cannot encode as Compact-U16"
        ));
    }
    let mut bytes = Vec::new();
    let mut rem_val = len as u16; // Safe to cast now
    loop {
        let mut elem = (rem_val & 0x7f) as u8;
        rem_val >>= 7;
        if rem_val == 0 {
            bytes.push(elem);
            break;
        } else {
            elem |= 0x80; // More bytes to follow, set MSB
            bytes.push(elem);
        }
    }
    Ok(bytes)
}

/// Decode a compact-u16 from the start of `bytes`, returning `(value, bytes_consumed)`.
///
/// Rejects truncated input, non-canonical encodings (a zero continuation byte),
/// and values above `u16::MAX`, matching `solana-short-vec`.
pub fn decode_compact_u16_len(bytes: &[u8]) -> Result<(usize, usize), &'static str> {
    const TRUNCATED: &str = "compact-u16 is truncated";
    const ALIAS: &str = "compact-u16 is not canonically encoded";

    let b0 = *bytes.first().ok_or(TRUNCATED)?;
    if b0 < 0x80 {
        return Ok((b0 as usize, 1));
    }
    let b1 = *bytes.get(1).ok_or(TRUNCATED)?;
    if b1 == 0 {
        return Err(ALIAS);
    }
    let low = (b0 & 0x7f) as usize;
    if b1 < 0x80 {
        return Ok((low | (b1 as usize) << 7, 2));
    }
    let b2 = *bytes.get(2).ok_or(TRUNCATED)?;
    if b2 == 0 {
        return Err(ALIAS);
    }
    // The third byte carries bits 14..16 and never has a continuation bit.
    if b2 > 3 {
        return Err("compact-u16 exceeds u16::MAX");
    }
    Ok((low | ((b1 & 0x7f) as usize) << 7 | (b2 as usize) << 14, 3))
}

/// If you don't want to use the ShortVec newtype, you can do ShortVec
/// deserialization on an ordinary vector with the following field annotation:
///
/// #[serde(with = "short_vec::deserialize")]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_seq(ShortVecVisitor { _t: PhantomData })
}

/// A newtype to provide Compact-U16 (AKA short_vec) serialization for `Vec<T>`
impl<T> Serialize for ShortVec<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Calls the module-level serialize function
        self::serialize(&self.inner, serializer)
    }
}

impl<'de, T> Deserialize<'de> for ShortVec<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Calls the module-level deserialize function
        Ok(ShortVec {
            inner: self::deserialize(deserializer)?,
        })
    }
}

#[derive(BorshSerialize, BorshDeserialize)] // Derives for Borsh
pub struct ShortVec<T> {
    pub inner: Vec<T>,
}

// Manual impls for common traits, forwarding to Vec<T>
// We need to be careful with bounds if T itself is complex.
impl<T: Clone> Clone for ShortVec<T> {
    fn clone(&self) -> Self {
        ShortVec {
            inner: self.inner.clone(),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ShortVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ShortVec").field(&self.inner).finish()
    }
}

impl<T: PartialEq> PartialEq for ShortVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

// Add a constructor and a way to get the inner Vec
impl<T> ShortVec<T> {
    pub fn new(inner: Vec<T>) -> Self {
        ShortVec { inner }
    }

    pub fn into_inner(self) -> Vec<T> {
        self.inner
    }

    // Optional: provide a way to borrow the inner vec
    pub fn as_inner(&self) -> &Vec<T> {
        &self.inner
    }

    pub fn as_mut_inner(&mut self) -> &mut Vec<T> {
        &mut self.inner
    }
}

// We still need to ensure T itself is bound correctly where ShortVec<T> is used.
// For Borsh: T must be BorshSerialize + BorshDeserialize.
// For Serde (via our custom impls): T must be Serialize + Deserialize<'de>.
// The derive for BorshSerialize/Deserialize on ShortVec<T> will require T to also implement them for Vec<T>.

#[cfg(test)]
mod tests {
    use super::*;
    use hexlit::hex;

    // Encodings from `solana-short-vec` 3.3.
    const ENCODINGS: [(u16, &[u8]); 10] = [
        (0, &hex!("00")),
        (1, &hex!("01")),
        (127, &hex!("7f")),
        (128, &hex!("8001")),
        (255, &hex!("ff01")),
        (256, &hex!("8002")),
        (16383, &hex!("ff7f")),
        (16384, &hex!("808001")),
        (32767, &hex!("ffff01")),
        (65535, &hex!("ffff03")),
    ];

    #[test]
    fn encode_matches_upstream() {
        for (value, bytes) in ENCODINGS {
            assert_eq!(
                encode_length_to_compact_u16_bytes(value as usize).unwrap(),
                bytes
            );
        }
        assert!(encode_length_to_compact_u16_bytes(u16::MAX as usize + 1).is_err());
    }

    #[test]
    fn decode_matches_upstream() {
        for (value, bytes) in ENCODINGS {
            assert_eq!(
                decode_compact_u16_len(bytes).unwrap(),
                (value as usize, bytes.len())
            );
            // Trailing input is left for the caller.
            let mut padded = bytes.to_vec();
            padded.push(0xff);
            assert_eq!(
                decode_compact_u16_len(&padded).unwrap(),
                (value as usize, bytes.len())
            );
        }
    }

    #[test]
    fn decode_rejects_what_upstream_rejects() {
        let rejected: [&[u8]; 9] = [
            &[],
            &hex!("80"),
            &hex!("8080"),
            // aliases: a zero continuation byte
            &hex!("8000"),
            &hex!("8100"),
            &hex!("808000"),
            // overflow / continuation on the third byte
            &hex!("ffff04"),
            &hex!("808080"),
            &hex!("ffff7f"),
        ];
        for bytes in rejected {
            assert!(
                decode_compact_u16_len(bytes).is_err(),
                "{bytes:02x?} must be rejected"
            );
        }
    }
}
