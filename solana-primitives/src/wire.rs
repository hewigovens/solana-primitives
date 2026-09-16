//! Solana wire format codecs.
//!
//! Legacy and v0 transactions are `compact-u16 signature count || signatures ||
//! message`. A v0 message is the legacy body prefixed with `0x80` and followed
//! by address table lookups. A v1 transaction is `0x81 || message ||
//! signatures`, with fixed-width counts (see [`crate::types::v1`]). Decoding is
//! strict: every length must be canonical and the input must be consumed exactly.

use crate::error::{DecodeError, EncodeError};
use crate::short_vec;
use crate::types::v1::{self, MessageV1, TransactionConfig, TransactionConfigMask};
use crate::types::{
    CompiledInstruction, Message, MessageAddressTableLookup, MessageHeader, MessageV0, Pubkey,
    SignatureBytes, VersionedMessage, VersionedTransaction,
};

/// High bit of the first message byte marks a versioned message.
pub(crate) const MESSAGE_VERSION_PREFIX: u8 = 0x80;

pub(crate) const SIGNATURE_LEN: usize = 64;
const PUBKEY_LEN: usize = 32;

/// A bounds-checked cursor over wire bytes.
pub(crate) struct WireReader<'a> {
    bytes: &'a [u8],
}

impl<'a> WireReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len()
    }

    pub fn peek_u8(&self) -> Result<u8, DecodeError> {
        self.bytes
            .first()
            .copied()
            .ok_or(DecodeError::UnexpectedEof)
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let (head, tail) = self
            .bytes
            .split_at_checked(len)
            .ok_or(DecodeError::UnexpectedEof)?;
        self.bytes = tail;
        Ok(head)
    }

    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let (head, tail) = self
            .bytes
            .split_first_chunk()
            .ok_or(DecodeError::UnexpectedEof)?;
        self.bytes = tail;
        Ok(*head)
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        self.read_array::<1>().map(|[byte]| byte)
    }

    pub fn read_u16_le(&mut self) -> Result<u16, DecodeError> {
        self.read_array().map(u16::from_le_bytes)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, DecodeError> {
        self.read_array().map(u32::from_le_bytes)
    }

    pub fn read_u64_le(&mut self) -> Result<u64, DecodeError> {
        self.read_array().map(u64::from_le_bytes)
    }

    pub fn read_short_u16(&mut self) -> Result<u16, DecodeError> {
        short_vec::decode(|| Ok(self.read_u8().ok()), |err| err)
    }

    /// Read a compact-u16 count of items that each occupy at least `min_item_len` bytes,
    /// rejecting counts the remaining input cannot hold before anything is allocated.
    pub fn read_count(&mut self, min_item_len: usize) -> Result<usize, DecodeError> {
        let count = usize::from(self.read_short_u16()?);
        if count * min_item_len > self.remaining() {
            return Err(DecodeError::UnexpectedEof);
        }
        Ok(count)
    }

    pub fn read_pubkeys(&mut self, count: usize) -> Result<Vec<Pubkey>, DecodeError> {
        let bytes = self.read_bytes(count * PUBKEY_LEN)?;
        Ok(bytes
            .as_chunks()
            .0
            .iter()
            .copied()
            .map(Pubkey::new)
            .collect())
    }

    pub fn read_signatures(&mut self, count: usize) -> Result<Vec<SignatureBytes>, DecodeError> {
        let bytes = self.read_bytes(count * SIGNATURE_LEN)?;
        Ok(bytes
            .as_chunks()
            .0
            .iter()
            .copied()
            .map(SignatureBytes::new)
            .collect())
    }

    /// Read a compact-u16 length-prefixed byte vector.
    pub fn read_short_vec_bytes(&mut self) -> Result<Vec<u8>, DecodeError> {
        let len = self.read_count(1)?;
        self.read_bytes(len).map(<[u8]>::to_vec)
    }

    pub fn finish(self) -> Result<(), DecodeError> {
        if self.bytes.is_empty() {
            Ok(())
        } else {
            Err(DecodeError::TrailingBytes)
        }
    }
}

/// Append a compact-u16 length.
pub(crate) fn write_short_len(out: &mut Vec<u8>, len: usize) -> Result<(), EncodeError> {
    let value = u16::try_from(len).map_err(|_| EncodeError::LengthOverflow {
        len,
        max: u16::MAX.into(),
    })?;
    let (bytes, n) = short_vec::encode(value);
    out.extend_from_slice(&bytes[..n]);
    Ok(())
}

fn write_short_vec_bytes(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), EncodeError> {
    write_short_len(out, bytes.len())?;
    out.extend_from_slice(bytes);
    Ok(())
}

pub(crate) fn write_header(out: &mut Vec<u8>, header: &MessageHeader) {
    out.extend_from_slice(&[
        header.num_required_signatures,
        header.num_readonly_signed_accounts,
        header.num_readonly_unsigned_accounts,
    ]);
}

pub(crate) fn write_pubkeys(out: &mut Vec<u8>, keys: &[Pubkey]) {
    for key in keys {
        out.extend_from_slice(key.as_bytes());
    }
}

pub(crate) fn read_header(reader: &mut WireReader) -> Result<MessageHeader, DecodeError> {
    let [
        num_required_signatures,
        num_readonly_signed_accounts,
        num_readonly_unsigned_accounts,
    ] = reader.read_array()?;
    Ok(MessageHeader {
        num_required_signatures,
        num_readonly_signed_accounts,
        num_readonly_unsigned_accounts,
    })
}

/// Header, account keys, blockhash, and instructions, shared by legacy and v0.
fn write_message_body(
    out: &mut Vec<u8>,
    header: &MessageHeader,
    account_keys: &[Pubkey],
    recent_blockhash: &[u8; 32],
    instructions: &[CompiledInstruction],
) -> Result<(), EncodeError> {
    write_header(out, header);
    write_short_len(out, account_keys.len())?;
    write_pubkeys(out, account_keys);
    out.extend_from_slice(recent_blockhash);
    write_short_len(out, instructions.len())?;
    for instruction in instructions {
        out.push(instruction.program_id_index);
        write_short_vec_bytes(out, &instruction.accounts)?;
        write_short_vec_bytes(out, &instruction.data)?;
    }
    Ok(())
}

fn read_message_body(reader: &mut WireReader) -> Result<Message, DecodeError> {
    let header = read_header(reader)?;
    let num_keys = reader.read_count(PUBKEY_LEN)?;
    let account_keys = reader.read_pubkeys(num_keys)?;
    let recent_blockhash = reader.read_array()?;
    // program index + account count + data length
    let num_instructions = reader.read_count(3)?;
    let instructions = (0..num_instructions)
        .map(|_| {
            Ok(CompiledInstruction {
                program_id_index: reader.read_u8()?,
                accounts: reader.read_short_vec_bytes()?,
                data: reader.read_short_vec_bytes()?,
            })
        })
        .collect::<Result<_, DecodeError>>()?;
    Ok(Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    })
}

pub(crate) fn write_legacy_message(
    out: &mut Vec<u8>,
    message: &Message,
) -> Result<(), EncodeError> {
    write_message_body(
        out,
        &message.header,
        &message.account_keys,
        &message.recent_blockhash,
        &message.instructions,
    )
}

pub(crate) fn write_v0_message(out: &mut Vec<u8>, message: &MessageV0) -> Result<(), EncodeError> {
    out.push(MESSAGE_VERSION_PREFIX);
    write_message_body(
        out,
        &message.header,
        &message.account_keys,
        &message.recent_blockhash,
        &message.instructions,
    )?;
    write_short_len(out, message.address_table_lookups.len())?;
    for lookup in &message.address_table_lookups {
        out.extend_from_slice(lookup.account_key.as_bytes());
        write_short_vec_bytes(out, &lookup.writable_indexes)?;
        write_short_vec_bytes(out, &lookup.readonly_indexes)?;
    }
    Ok(())
}

fn read_v0_message(reader: &mut WireReader) -> Result<MessageV0, DecodeError> {
    let body = read_message_body(reader)?;
    // key + two lengths
    let num_lookups = reader.read_count(PUBKEY_LEN + 2)?;
    let address_table_lookups = (0..num_lookups)
        .map(|_| {
            Ok(MessageAddressTableLookup {
                account_key: Pubkey::new(reader.read_array()?),
                writable_indexes: reader.read_short_vec_bytes()?,
                readonly_indexes: reader.read_short_vec_bytes()?,
            })
        })
        .collect::<Result<_, DecodeError>>()?;
    Ok(MessageV0 {
        header: body.header,
        account_keys: body.account_keys,
        recent_blockhash: body.recent_blockhash,
        instructions: body.instructions,
        address_table_lookups,
    })
}

fn u8_len(len: usize) -> Result<u8, EncodeError> {
    u8::try_from(len).map_err(|_| EncodeError::LengthOverflow {
        len,
        max: u8::MAX.into(),
    })
}

pub(crate) fn write_v1_message(out: &mut Vec<u8>, message: &MessageV1) -> Result<(), EncodeError> {
    out.push(v1::VERSION_PREFIX);
    write_header(out, &message.header);
    out.extend_from_slice(&message.config.mask().0.to_le_bytes());
    out.extend_from_slice(&message.lifetime_specifier);
    out.push(u8_len(message.instructions.len())?);
    out.push(u8_len(message.account_keys.len())?);
    write_pubkeys(out, &message.account_keys);

    // Values follow the mask's bit order.
    let config = &message.config;
    if let Some(fee) = config.priority_fee {
        out.extend_from_slice(&fee.to_le_bytes());
    }
    for value in [
        config.compute_unit_limit,
        config.loaded_accounts_data_size_limit,
        config.heap_size,
    ]
    .into_iter()
    .flatten()
    {
        out.extend_from_slice(&value.to_le_bytes());
    }

    // All fixed-size instruction headers come before any payload.
    for instruction in &message.instructions {
        let data_len =
            u16::try_from(instruction.data.len()).map_err(|_| EncodeError::LengthOverflow {
                len: instruction.data.len(),
                max: u16::MAX.into(),
            })?;
        out.push(instruction.program_id_index);
        out.push(u8_len(instruction.accounts.len())?);
        out.extend_from_slice(&data_len.to_le_bytes());
    }
    for instruction in &message.instructions {
        out.extend_from_slice(&instruction.accounts);
        out.extend_from_slice(&instruction.data);
    }
    Ok(())
}

/// Read a v1 message after its `0x81` prefix.
fn read_v1_message(reader: &mut WireReader) -> Result<MessageV1, DecodeError> {
    let header = read_header(reader)?;
    let mask = TransactionConfigMask(reader.read_u32_le()?);
    // Unknown bits could not be re-encoded, which would change the signed bytes.
    if !mask.is_valid() {
        return Err(DecodeError::InvalidConfigMask(mask.0));
    }
    let lifetime_specifier = reader.read_array()?;
    let num_instructions = reader.read_u8()?;
    let num_addresses = reader.read_u8()?;
    let account_keys = reader.read_pubkeys(num_addresses.into())?;

    let config = TransactionConfig {
        priority_fee: mask
            .has_priority_fee()
            .then(|| reader.read_u64_le())
            .transpose()?,
        compute_unit_limit: mask
            .has_compute_unit_limit()
            .then(|| reader.read_u32_le())
            .transpose()?,
        loaded_accounts_data_size_limit: mask
            .has_loaded_accounts_data_size_limit()
            .then(|| reader.read_u32_le())
            .transpose()?,
        heap_size: mask
            .has_heap_size()
            .then(|| reader.read_u32_le())
            .transpose()?,
    };

    let headers = (0..num_instructions)
        .map(|_| Ok((reader.read_u8()?, reader.read_u8()?, reader.read_u16_le()?)))
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let instructions = headers
        .into_iter()
        .map(|(program_id_index, num_accounts, data_len)| {
            Ok(CompiledInstruction {
                program_id_index,
                accounts: reader.read_bytes(num_accounts.into())?.to_vec(),
                data: reader.read_bytes(data_len.into())?.to_vec(),
            })
        })
        .collect::<Result<_, DecodeError>>()?;

    Ok(MessageV1 {
        header,
        config,
        lifetime_specifier,
        account_keys,
        instructions,
    })
}

pub(crate) fn write_message(
    out: &mut Vec<u8>,
    message: &VersionedMessage,
) -> Result<(), EncodeError> {
    match message {
        VersionedMessage::Legacy(message) => write_legacy_message(out, message),
        VersionedMessage::V0(message) => write_v0_message(out, message),
        VersionedMessage::V1(message) => write_v1_message(out, message),
    }
}

pub(crate) fn read_message(reader: &mut WireReader) -> Result<VersionedMessage, DecodeError> {
    let first = reader.peek_u8()?;
    if first & MESSAGE_VERSION_PREFIX == 0 {
        return read_message_body(reader).map(VersionedMessage::Legacy);
    }
    reader.read_u8()?;
    match first & !MESSAGE_VERSION_PREFIX {
        0 => read_v0_message(reader).map(VersionedMessage::V0),
        1 => read_v1_message(reader).map(VersionedMessage::V1),
        version => Err(DecodeError::UnsupportedMessageVersion(version)),
    }
}

/// Legacy/v0 envelope: one-byte signature count, signatures, then the message.
pub(crate) fn encode_legacy_envelope(
    signatures: &[SignatureBytes],
    write_message: impl FnOnce(&mut Vec<u8>) -> Result<(), EncodeError>,
) -> Result<Vec<u8>, EncodeError> {
    // The count must fit in one byte: a set high bit would read as a message version.
    let num_signatures = u8::try_from(signatures.len())
        .ok()
        .filter(|count| count & MESSAGE_VERSION_PREFIX == 0)
        .ok_or(EncodeError::LengthOverflow {
            len: signatures.len(),
            max: usize::from(MESSAGE_VERSION_PREFIX) - 1,
        })?;
    let mut out = Vec::with_capacity(1 + signatures.len() * SIGNATURE_LEN);
    out.push(num_signatures);
    for signature in signatures {
        out.extend_from_slice(signature.as_bytes());
    }
    write_message(&mut out)?;
    Ok(out)
}

/// v1 envelope: the message, then exactly `num_required_signatures` signatures.
fn encode_v1_envelope(
    signatures: &[SignatureBytes],
    message: &MessageV1,
) -> Result<Vec<u8>, EncodeError> {
    let expected = usize::from(message.header.num_required_signatures);
    if signatures.len() != expected {
        return Err(EncodeError::SignatureCountMismatch {
            expected,
            actual: signatures.len(),
        });
    }
    let mut out = Vec::new();
    write_v1_message(&mut out, message)?;
    for signature in signatures {
        out.extend_from_slice(signature.as_bytes());
    }
    Ok(out)
}

pub(crate) fn encode_transaction(
    transaction: &VersionedTransaction,
) -> Result<Vec<u8>, EncodeError> {
    match &transaction.message {
        VersionedMessage::V1(message) => encode_v1_envelope(&transaction.signatures, message),
        message => {
            encode_legacy_envelope(&transaction.signatures, |out| write_message(out, message))
        }
    }
}

pub(crate) fn decode_transaction(bytes: &[u8]) -> Result<VersionedTransaction, DecodeError> {
    let mut reader = WireReader::new(bytes);
    let discriminator = reader.read_u8()?;
    if discriminator == v1::VERSION_PREFIX {
        // The signature count comes from the message header.
        let message = read_v1_message(&mut reader)?;
        let signatures = reader.read_signatures(message.header.num_required_signatures.into())?;
        reader.finish()?;
        return Ok(VersionedTransaction {
            signatures,
            message: VersionedMessage::V1(message),
        });
    }
    if discriminator & MESSAGE_VERSION_PREFIX != 0 {
        return Err(DecodeError::InvalidTransactionDiscriminator(discriminator));
    }
    // With the high bit clear, the byte is a complete one-byte compact-u16.
    let signatures = reader.read_signatures(discriminator.into())?;
    let message = read_message(&mut reader)?;
    if matches!(message, VersionedMessage::V1(_)) {
        return Err(DecodeError::UnexpectedVersion);
    }
    reader.finish()?;
    Ok(VersionedTransaction {
        signatures,
        message,
    })
}

/// Decode exactly one message from `bytes`.
pub(crate) fn decode_message(bytes: &[u8]) -> Result<VersionedMessage, DecodeError> {
    let mut reader = WireReader::new(bytes);
    let message = read_message(&mut reader)?;
    reader.finish()?;
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_primitives() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut reader = WireReader::new(&bytes);
        assert_eq!(reader.read_u8(), Ok(1));
        assert_eq!(reader.read_array(), Ok([2, 3]));
        assert_eq!(reader.read_array::<6>(), Err(DecodeError::UnexpectedEof));
        assert_eq!(reader.read_bytes(6), Err(DecodeError::UnexpectedEof));
        assert_eq!(reader.read_bytes(5), Ok(&bytes[3..]));
        assert_eq!(reader.finish(), Ok(()));

        let mut reader = WireReader::new(&[0xff, 0xff, 0x03, 0]);
        assert_eq!(reader.read_short_u16(), Ok(u16::MAX));
        assert_eq!(reader.finish(), Err(DecodeError::TrailingBytes));
    }

    #[test]
    fn read_count_rejects_counts_larger_than_input() {
        let mut reader = WireReader::new(&[3, 0, 0]);
        assert_eq!(reader.read_count(1), Err(DecodeError::UnexpectedEof));
        let mut reader = WireReader::new(&[2, 0, 0]);
        assert_eq!(reader.read_count(1), Ok(2));
    }

    #[test]
    fn signature_count_must_fit_one_byte() {
        let transaction = VersionedTransaction {
            signatures: vec![SignatureBytes::default(); 128],
            message: VersionedMessage::Legacy(Message::default()),
        };
        assert_eq!(
            encode_transaction(&transaction),
            Err(EncodeError::LengthOverflow { len: 128, max: 127 })
        );
    }
}
