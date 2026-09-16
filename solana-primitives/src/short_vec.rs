//! Compact-u16 ("shortvec") length encoding.
//!
//! A `u16` is written in 1–3 bytes, 7 bits at a time from the least significant
//! end, with the high bit of each byte signalling that another byte follows. The
//! third byte carries the top two bits and never has a continuation bit.
//! Decoding is canonical: a zero continuation byte (an alias of a shorter
//! encoding) and values above `u16::MAX` are rejected, as in `solana-short-vec`.

use crate::error::DecodeError;

const MAX_ENCODED_LEN: usize = 3;

/// Encode `value`, returning the buffer and the number of bytes used.
pub(crate) fn encode(value: u16) -> ([u8; MAX_ENCODED_LEN], usize) {
    let mut out = [0u8; MAX_ENCODED_LEN];
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

/// Decode a compact-u16 from the start of `bytes`, returning the value and the
/// number of bytes it occupied.
pub(crate) fn decode(bytes: &[u8]) -> Result<(u16, usize), DecodeError> {
    let byte = |index: usize| bytes.get(index).copied().ok_or(DecodeError::UnexpectedEof);

    let b0 = byte(0)?;
    if b0 < 0x80 {
        return Ok((b0.into(), 1));
    }
    let low = u16::from(b0 & 0x7f);
    let b1 = byte(1)?;
    if b1 == 0 {
        return Err(DecodeError::NonCanonicalCompactU16);
    }
    if b1 < 0x80 {
        return Ok((low | u16::from(b1) << 7, 2));
    }
    let b2 = byte(2)?;
    if b2 == 0 {
        return Err(DecodeError::NonCanonicalCompactU16);
    }
    if b2 > 3 {
        return Err(DecodeError::CompactU16Overflow);
    }
    Ok((low | u16::from(b1 & 0x7f) << 7 | u16::from(b2) << 14, 3))
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
            let (encoded, len) = encode(value);
            assert_eq!(&encoded[..len], bytes);
        }
    }

    #[test]
    fn decode_matches_upstream() {
        for (value, bytes) in ENCODINGS {
            assert_eq!(decode(bytes), Ok((value, bytes.len())));
            // Trailing input is left for the caller.
            let padded = [bytes, &[0xff]].concat();
            assert_eq!(decode(&padded), Ok((value, bytes.len())));
        }
    }

    #[test]
    fn decode_rejects_what_upstream_rejects() {
        let rejected: [(&[u8], DecodeError); 9] = [
            (&[], DecodeError::UnexpectedEof),
            (&hex!("80"), DecodeError::UnexpectedEof),
            (&hex!("8080"), DecodeError::UnexpectedEof),
            (&hex!("8000"), DecodeError::NonCanonicalCompactU16),
            (&hex!("8100"), DecodeError::NonCanonicalCompactU16),
            (&hex!("808000"), DecodeError::NonCanonicalCompactU16),
            (&hex!("ffff04"), DecodeError::CompactU16Overflow),
            (&hex!("808080"), DecodeError::CompactU16Overflow),
            (&hex!("ffff7f"), DecodeError::CompactU16Overflow),
        ];
        for (bytes, err) in rejected {
            assert_eq!(decode(bytes), Err(err), "{bytes:02x?}");
        }
    }
}
