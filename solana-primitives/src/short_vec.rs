//! Compact-u16 ("shortvec") length encoding.
//!
//! A `u16` is written in 1–3 bytes, 7 bits at a time from the least significant
//! end, with the high bit of each byte signalling that another byte follows. The
//! third byte carries the top two bits and never has a continuation bit.
//! Decoding is canonical: a zero continuation byte (an alias of a shorter
//! encoding) and values above `u16::MAX` are rejected, as in `solana-short-vec`.

use crate::error::{DecodeError, EncodeError};
#[cfg(feature = "serde")]
use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, SeqAccess, Visitor},
    ser::{self, SerializeTuple, Serializer},
};
#[cfg(feature = "serde")]
use std::{fmt, marker::PhantomData};

/// Maximum encoded length of a compact-u16.
pub const MAX_ENCODING_LENGTH: usize = 3;

/// Encode `value`, returning the buffer and the number of bytes used.
pub(crate) fn encode(value: u16) -> ([u8; MAX_ENCODING_LENGTH], usize) {
    let mut out = [0u8; MAX_ENCODING_LENGTH];
    let mut rem = value;
    let mut len = 0;
    loop {
        let byte = (rem & 0x7f) as u8;
        rem >>= 7;
        if rem == 0 {
            out[len] = byte;
            return (out, len + 1);
        }
        out[len] = byte | 0x80;
        len += 1;
    }
}

/// Decode a compact-u16, pulling bytes from `next_byte` (`Ok(None)` means end of input).
pub(crate) fn decode<E>(
    mut next_byte: impl FnMut() -> Result<Option<u8>, E>,
    error: impl Fn(DecodeError) -> E,
) -> Result<u16, E> {
    let mut next = || next_byte()?.ok_or_else(|| error(DecodeError::UnexpectedEof));

    let b0 = next()?;
    if b0 < 0x80 {
        return Ok(b0.into());
    }
    let low = u16::from(b0 & 0x7f);
    let b1 = next()?;
    if b1 == 0 {
        return Err(error(DecodeError::NonCanonicalShortU16));
    }
    if b1 < 0x80 {
        return Ok(low | u16::from(b1) << 7);
    }
    let b2 = next()?;
    if b2 == 0 {
        return Err(error(DecodeError::NonCanonicalShortU16));
    }
    if b2 > 3 {
        return Err(error(DecodeError::ShortU16Overflow));
    }
    Ok(low | u16::from(b1 & 0x7f) << 7 | u16::from(b2) << 14)
}

/// Encode a length as compact-u16 bytes.
pub fn encode_length_to_compact_u16_bytes(len: usize) -> Result<Vec<u8>, EncodeError> {
    let value = u16::try_from(len).map_err(|_| EncodeError::LengthOverflow {
        len,
        max: u16::MAX.into(),
    })?;
    let (bytes, n) = encode(value);
    Ok(bytes[..n].to_vec())
}

/// Decode a compact-u16 from the start of `bytes`, returning `(value, bytes_consumed)`.
pub fn decode_compact_u16_len(bytes: &[u8]) -> Result<(usize, usize), DecodeError> {
    let mut consumed = 0;
    let value = decode(
        || {
            let byte = bytes.get(consumed).copied();
            consumed += 1;
            Ok(byte)
        },
        |err| err,
    )?;
    Ok((value.into(), consumed))
}

/// A `u16` serialized with serde as a compact-u16 byte tuple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortU16(pub u16);

#[cfg(feature = "serde")]
impl Serialize for ShortU16 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (bytes, len) = encode(self.0);
        // A non-zero tuple length makes serde_json emit brackets.
        let mut seq = serializer.serialize_tuple(1)?;
        for byte in &bytes[..len] {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ShortU16 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ShortU16Visitor;

        impl<'de> Visitor<'de> for ShortU16Visitor {
            type Value = ShortU16;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a compact-u16")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<ShortU16, A::Error> {
                decode(|| seq.next_element::<u8>(), de::Error::custom).map(ShortU16)
            }
        }

        deserializer.deserialize_tuple(MAX_ENCODING_LENGTH, ShortU16Visitor)
    }
}

/// Serialize a slice as a compact-u16 length followed by its elements.
///
/// Use with `#[serde(with = "solana_primitives::short_vec")]`.
#[cfg(feature = "serde")]
pub fn serialize<S: Serializer, T: Serialize>(
    elements: &[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let len =
        u16::try_from(elements.len()).map_err(|_| ser::Error::custom("length larger than u16"))?;
    // A non-zero tuple length makes serde_json emit brackets.
    let mut seq = serializer.serialize_tuple(1)?;
    seq.serialize_element(&ShortU16(len))?;
    for element in elements {
        seq.serialize_element(element)?;
    }
    seq.end()
}

/// Deserialize a compact-u16 length followed by that many elements.
///
/// Use with `#[serde(with = "solana_primitives::short_vec")]`.
#[cfg(feature = "serde")]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ShortVecVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for ShortVecVisitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a Vec with a compact-u16 length")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<T>, A::Error> {
            let ShortU16(len) = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let len = usize::from(len);
            // Cap the reservation so a malformed length can't allocate much up front.
            let mut result = Vec::with_capacity(len.min(1024));
            for i in 0..len {
                let element = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(i, &self))?;
                result.push(element);
            }
            Ok(result)
        }
    }

    deserializer.deserialize_tuple(usize::MAX, ShortVecVisitor(PhantomData))
}

/// A `Vec<T>` serialized with serde using a compact-u16 length prefix.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShortVec<T>(pub Vec<T>);

#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for ShortVec<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize(&self.0, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for ShortVec<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserialize(deserializer).map(ShortVec)
    }
}

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
                encode_length_to_compact_u16_bytes(value.into()).unwrap(),
                bytes
            );
        }
        assert_eq!(
            encode_length_to_compact_u16_bytes(65536),
            Err(EncodeError::LengthOverflow {
                len: 65536,
                max: 65535
            })
        );
    }

    #[test]
    fn decode_matches_upstream() {
        for (value, bytes) in ENCODINGS {
            assert_eq!(
                decode_compact_u16_len(bytes),
                Ok((value.into(), bytes.len()))
            );
            // Trailing input is left for the caller.
            let padded = [bytes, &[0xff]].concat();
            assert_eq!(
                decode_compact_u16_len(&padded),
                Ok((value.into(), bytes.len()))
            );
        }
    }

    #[test]
    fn decode_rejects_what_upstream_rejects() {
        let rejected: [(&[u8], DecodeError); 9] = [
            (&[], DecodeError::UnexpectedEof),
            (&hex!("80"), DecodeError::UnexpectedEof),
            (&hex!("8080"), DecodeError::UnexpectedEof),
            (&hex!("8000"), DecodeError::NonCanonicalShortU16),
            (&hex!("8100"), DecodeError::NonCanonicalShortU16),
            (&hex!("808000"), DecodeError::NonCanonicalShortU16),
            (&hex!("ffff04"), DecodeError::ShortU16Overflow),
            (&hex!("808080"), DecodeError::ShortU16Overflow),
            (&hex!("ffff7f"), DecodeError::ShortU16Overflow),
        ];
        for (bytes, err) in rejected {
            assert_eq!(decode_compact_u16_len(bytes), Err(err), "{bytes:02x?}");
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_uses_the_same_encoding() {
        for (value, bytes) in ENCODINGS {
            let json = serde_json::to_string(&ShortU16(value)).unwrap();
            assert_eq!(serde_json::from_str::<Vec<u8>>(&json).unwrap(), bytes);
            assert_eq!(
                serde_json::from_str::<ShortU16>(&json).unwrap(),
                ShortU16(value)
            );
        }
        assert!(serde_json::from_str::<ShortU16>("[128, 0]").is_err());

        // Self-describing formats nest the length tuple, as upstream does.
        let vec = ShortVec(vec![1u8, 2, 3]);
        assert_eq!(serde_json::to_string(&vec).unwrap(), "[[3],1,2,3]");
        assert_eq!(
            serde_json::from_str::<ShortVec<u8>>("[[3],1,2,3]").unwrap(),
            vec
        );
    }
}
