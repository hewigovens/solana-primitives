//! Error types.

use crate::types::Pubkey;
use std::fmt;

/// Errors returned by this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SolanaError {
    /// A string is not valid base58.
    InvalidBase58,
    /// A byte slice has the wrong length for the target type.
    InvalidLength { expected: usize, actual: usize },
    /// Public key bytes are not a valid Ed25519 point.
    InvalidPublicKey,
    /// A signature does not verify against its signer and message.
    InvalidSignature,
    /// A private key does not belong to any of the message's required signers.
    UnexpectedSigner(Pubkey),
    /// A required signer has not signed.
    MissingSigner(Pubkey),
    /// PDA seeds are too many or too long, or the derived address is on the curve.
    InvalidSeeds,
    /// No bump seed produces an off-curve program address.
    NoViableBumpSeed,
    /// `create_with_seed` owner ends with the program-derived-address marker.
    IllegalOwner,
    /// Instructions cannot be compiled into a message.
    Compile(CompileError),
    /// Wire bytes are malformed.
    Decode(DecodeError),
    /// A value cannot be represented in the wire format.
    Encode(EncodeError),
    /// A message or transaction violates Solana's sanitization rules.
    Sanitize(SanitizeError),
    /// A serialized transaction exceeds its version's size limit.
    TransactionTooLarge { size: usize, max: usize },
    /// The operation is not available for this transaction version.
    UnsupportedVersion,
    /// On-chain account data is malformed.
    InvalidAccountData,
}

impl fmt::Display for SolanaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBase58 => f.write_str("invalid base58 string"),
            Self::InvalidLength { expected, actual } => {
                write!(f, "invalid length: expected {expected} bytes, got {actual}")
            }
            Self::InvalidPublicKey => f.write_str("public key is not a valid ed25519 point"),
            Self::InvalidSignature => f.write_str("signature verification failed"),
            Self::UnexpectedSigner(key) => write!(f, "{key} is not a required signer"),
            Self::MissingSigner(key) => write!(f, "missing signature for {key}"),
            Self::InvalidSeeds => f.write_str("invalid program address seeds"),
            Self::NoViableBumpSeed => f.write_str("no viable program address bump seed"),
            Self::IllegalOwner => f.write_str("owner must not end with the PDA marker"),
            Self::Compile(err) => write!(f, "compile error: {err}"),
            Self::Decode(err) => write!(f, "decode error: {err}"),
            Self::Encode(err) => write!(f, "encode error: {err}"),
            Self::Sanitize(err) => write!(f, "sanitize error: {err}"),
            Self::TransactionTooLarge { size, max } => {
                write!(f, "transaction is {size} bytes, max is {max}")
            }
            Self::UnsupportedVersion => {
                f.write_str("operation not supported for this transaction version")
            }
            Self::InvalidAccountData => f.write_str("invalid account data"),
        }
    }
}

impl std::error::Error for SolanaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Compile(err) => Some(err),
            Self::Decode(err) => Some(err),
            Self::Encode(err) => Some(err),
            Self::Sanitize(err) => Some(err),
            _ => None,
        }
    }
}

/// Errors from compiling instructions into a message.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompileError {
    /// More accounts than a `u8` index can address, or a header count overflowed.
    AccountIndexOverflow,
    /// An address lookup table entry index does not fit in a `u8`.
    AddressTableLookupIndexOverflow,
    /// An instruction references a key that was not compiled.
    UnknownInstructionKey(Pubkey),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AccountIndexOverflow => f.write_str("account index overflowed"),
            Self::AddressTableLookupIndexOverflow => {
                f.write_str("address lookup table index overflowed")
            }
            Self::UnknownInstructionKey(key) => write!(f, "unknown instruction key {key}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// Errors from decoding wire bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// The input ended early.
    UnexpectedEof,
    /// Bytes remain after a complete value.
    TrailingBytes,
    /// A compact-u16 has a zero continuation byte.
    NonCanonicalCompactU16,
    /// A compact-u16 exceeds `u16::MAX`.
    CompactU16Overflow,
    /// The first transaction byte is neither a legacy/v0 signature count nor a v1 prefix.
    InvalidTransactionDiscriminator(u8),
    /// The message version is not supported.
    UnsupportedMessageVersion(u8),
    /// The bytes are a valid transaction of a different version than requested.
    UnexpectedVersion,
    /// A v1 config mask has unknown bits or half of the priority-fee bit pair.
    InvalidConfigMask(u32),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected end of input"),
            Self::TrailingBytes => f.write_str("trailing bytes after value"),
            Self::NonCanonicalCompactU16 => f.write_str("non-canonical compact-u16"),
            Self::CompactU16Overflow => f.write_str("compact-u16 exceeds u16::MAX"),
            Self::InvalidTransactionDiscriminator(byte) => {
                write!(f, "invalid transaction discriminator {byte:#04x}")
            }
            Self::UnsupportedMessageVersion(version) => {
                write!(f, "unsupported message version {version}")
            }
            Self::UnexpectedVersion => f.write_str("unexpected transaction version"),
            Self::InvalidConfigMask(mask) => write!(f, "invalid transaction config mask {mask:#x}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Errors from encoding values into wire bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EncodeError {
    /// A length does not fit its wire field.
    LengthOverflow { len: usize, max: usize },
    /// A v1 transaction must carry exactly `num_required_signatures` signatures.
    SignatureCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow { len, max } => {
                write!(f, "length {len} exceeds wire maximum {max}")
            }
            Self::SignatureCountMismatch { expected, actual } => {
                write!(f, "expected {expected} signatures, got {actual}")
            }
        }
    }
}

impl std::error::Error for EncodeError {}

/// Violations of Solana's message and transaction sanitization rules.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SanitizeError {
    /// The header describes more signers or readonly accounts than there are keys.
    NotEnoughAccountKeys,
    /// Every signer is readonly, so there is no writable fee payer.
    NoWritableFeePayer,
    /// A program index is out of range, points at the fee payer, or at a looked-up key.
    InvalidProgramIndex,
    /// An instruction account index is out of range.
    InvalidAccountIndex,
    /// An address table lookup loads no accounts.
    EmptyAddressTableLookup,
    /// The message references more accounts than its version allows.
    TooManyAccountKeys,
    /// The message has more instructions than its version allows.
    TooManyInstructions,
    /// The message requires more signatures than its version allows.
    TooManySignatures,
    /// An account key appears more than once.
    DuplicateAccountKeys,
    /// An instruction has more accounts than its version can encode.
    InstructionAccountsTooLarge,
    /// An instruction has more data than its version can encode.
    InstructionDataTooLarge,
    /// A requested heap size is not a multiple of 1 KiB within 32..=256 KiB.
    InvalidHeapSize,
    /// The number of signatures differs from `num_required_signatures`.
    SignatureCountMismatch { expected: usize, actual: usize },
}

impl fmt::Display for SanitizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotEnoughAccountKeys => f.write_str("header describes more accounts than keys"),
            Self::NoWritableFeePayer => f.write_str("no writable fee payer"),
            Self::InvalidProgramIndex => f.write_str("invalid program index"),
            Self::InvalidAccountIndex => f.write_str("invalid account index"),
            Self::EmptyAddressTableLookup => f.write_str("address table lookup loads no accounts"),
            Self::TooManyAccountKeys => f.write_str("too many account keys"),
            Self::TooManyInstructions => f.write_str("too many instructions"),
            Self::TooManySignatures => f.write_str("too many signatures"),
            Self::DuplicateAccountKeys => f.write_str("duplicate account keys"),
            Self::InstructionAccountsTooLarge => f.write_str("instruction has too many accounts"),
            Self::InstructionDataTooLarge => f.write_str("instruction data is too large"),
            Self::InvalidHeapSize => f.write_str("invalid heap size"),
            Self::SignatureCountMismatch { expected, actual } => {
                write!(f, "expected {expected} signatures, got {actual}")
            }
        }
    }
}

impl std::error::Error for SanitizeError {}

impl From<CompileError> for SolanaError {
    fn from(err: CompileError) -> Self {
        Self::Compile(err)
    }
}

impl From<DecodeError> for SolanaError {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl From<EncodeError> for SolanaError {
    fn from(err: EncodeError) -> Self {
        Self::Encode(err)
    }
}

impl From<SanitizeError> for SolanaError {
    fn from(err: SanitizeError) -> Self {
        Self::Sanitize(err)
    }
}

/// A [`std::result::Result`] with [`SolanaError`] as the error type.
pub type Result<T> = std::result::Result<T, SolanaError>;
