mod account;
pub mod instruction;
mod message;
mod pda;
mod pubkey;
mod signature;
mod transaction;

pub use crate::error::{Result, SolanaError};
pub use account::{AddressLookupTableAccount, MessageAddressTableLookup};
pub use instruction::{AccountMeta, CompiledInstruction, Instruction};
pub use message::{LegacyMessage, Message, MessageHeader, VersionedMessage, VersionedMessageV0};
pub use pda::{create_program_address, find_program_address};
pub use pubkey::Pubkey;
pub use signature::SignatureBytes;
pub use transaction::{Transaction, VersionedTransaction};

// Constants
/// Maximum allowed size for a Solana transaction in bytes
pub const MAX_TRANSACTION_SIZE: usize = 1232;
