# Solana Primitives

[![CI](https://github.com/hewigovens/solana-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/solana-primitives/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/solana-primitives.svg)](https://crates.io/crates/solana-primitives)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/solana-primitives)
[![License](https://img.shields.io/badge/License-Apache--2.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

A lightweight Rust crate for building, signing, serializing, and parsing Solana transactions without the full Solana SDK.

## Features

- **All transaction versions**: legacy, v0 (address lookup tables), and v1 ([SIMD-0385](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0385-transaction-v1.md): transactions up to 4096 bytes, compute budget in the message config)
- **Core types**: `Pubkey`, `SignatureBytes`, `Instruction`, `AccountMeta`, `Message`, `MessageV0`, `MessageV1`, `VersionedMessage`, `Transaction`, `VersionedTransaction`
- **Builders**: `TransactionBuilder` merges account roles and orders keys the same way the Solana SDK does; `InstructionBuilder` and `InstructionDataBuilder` for custom instructions
- **Strict wire codec**: canonical compact-u16 lengths, no trailing bytes, per-version size limits, and Solana's sanitization rules on every `deserialize`
- **Signing and verification** for every version
- **Program helpers**: System, SPL Token (and Token-2022), Associated Token Account, Compute Budget, Memo, Anchor discriminators
- **PDAs**: `find_program_address`, `create_program_address`, `create_with_seed`
- **Verified encodings**: golden vectors generated with the Solana SDK (`solana-message` 5.0, `solana-transaction` 5.0, `solana-system-interface`, `spl-token-interface`, `spl-associated-token-account-interface`)
- **Small dependency footprint**: `ed25519-dalek`, `bs58`, and `sha2` by default

## Usage

```toml
[dependencies]
solana-primitives = "0.3"
```

Optional features:

| Feature | Adds |
|---|---|
| `serde` | Serde for the Rust data model (pubkeys and signatures as base58 strings). Not the transaction wire format. |
| `borsh` | Borsh for `Pubkey` and `SignatureBytes`. Not the transaction wire format. |

Use `serialize()` / `deserialize()` for the bytes you send to or receive from the network.

### Legacy transaction

```rust
use solana_primitives::{
    InstructionBuilder, InstructionDataBuilder, Pubkey, TransactionBuilder, get_public_key,
    instructions::{program_ids::system_program, system::transfer},
};

fn main() -> solana_primitives::Result<()> {
    let private_key = [7u8; 32];
    let fee_payer = Pubkey::new(get_public_key(&private_key)?);
    let recipient = Pubkey::from_base58("4fYNw3dojWmQ4dXtSGE9epjRGy9uFrCRgbvGgQBNZCQF")?;
    let recent_blockhash = [0u8; 32]; // from `getLatestBlockhash`

    // A pre-built instruction...
    let transfer_instruction = transfer(&fee_payer, &recipient, 1_000_000);

    // ...or a hand-built one. System instructions use a u32 discriminant.
    let custom_instruction = InstructionBuilder::new(system_program())
        .account(fee_payer, true, true)
        .account(recipient, false, true)
        .data(InstructionDataBuilder::new().u32(2).u64(1_000_000).build())
        .build();
    assert_eq!(custom_instruction, transfer_instruction);

    let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    builder.add_instruction(transfer_instruction);
    let mut transaction = builder.build()?;

    transaction.sign(&[&private_key])?;
    transaction.validate_size()?;
    let wire_bytes = transaction.serialize()?; // base64-encode for `sendTransaction`
    Ok(())
}
```

### v1 transaction

v1 carries compute budget requests in `TransactionConfig`; Compute Budget instructions are ignored by the runtime. Unset values mean **zero** (heap defaults to 32 KiB), so set the compute unit limit and the loaded accounts data size limit explicitly. The priority fee is a **total in lamports**, not a per-unit price.

```rust
use solana_primitives::{TransactionBuilder, TransactionConfig};

let config = TransactionConfig::new()
    .with_compute_unit_limit(200_000)
    .with_loaded_accounts_data_size_limit(64 * 1024)
    .with_priority_fee(5_000);
let mut transaction = builder.build_v1(config)?;
transaction.sign(&[&private_key])?;
let wire_bytes = transaction.serialize()?; // `0x81 || message || signatures`
```

### v0 transaction with lookup tables

```rust
use solana_primitives::AddressLookupTableAccount;

// Parse lookup tables fetched with `getAccountInfo`...
let table = AddressLookupTableAccount::from_account_data(table_key, &account_data)?;
// ...or construct them directly.
let tables = vec![AddressLookupTableAccount::new(table_key, vec![looked_up])];

let mut transaction = builder.build_v0(&tables)?;
transaction.sign(&[&private_key])?;
```

Signers, invoked programs, and a durable nonce account always stay in the static keys.

### Parsing and inspecting transactions

```rust
use solana_primitives::{TransactionVersion, VersionedTransaction};

let transaction = VersionedTransaction::deserialize(&wire_bytes)?;
match transaction.version() {
    TransactionVersion::V1 => println!("fee: {:?} lamports", transaction.priority_fee_lamports()),
    _ => println!("price: {:?} micro-lamports/CU", transaction.get_compute_unit_price()),
}
println!("CU limit: {:?}", transaction.get_compute_unit_limit());
transaction.verify()?;
```

Compute budget getters report what the runtime would apply: they return `None` when the Compute Budget instructions would fail the transaction (an unparsable or repeated request). Setters clear signatures when they change the message.

`deserialize` rejects truncated input, trailing bytes, non-canonical lengths, unknown versions and config bits, oversized transactions (1232 bytes for legacy/v0, 4096 for v1), and anything that fails `sanitize()`.

### Program Derived Addresses

```rust
use solana_primitives::{Pubkey, find_program_address};

let program_id = Pubkey::from_base58("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
let (pda, bump) = find_program_address(&program_id, &[b"vault", owner.as_bytes()])?;
```

### Error handling

Errors are a typed `SolanaError` with `Compile`, `Decode`, `Encode`, and `Sanitize` sub-errors:

```rust
use solana_primitives::{DecodeError, SolanaError, VersionedTransaction};

match VersionedTransaction::deserialize(&bytes) {
    Err(SolanaError::Decode(DecodeError::TrailingBytes)) => { /* ... */ }
    Err(SolanaError::Sanitize(err)) => eprintln!("invalid transaction: {err}"),
    Err(err) => eprintln!("{err}"),
    Ok(transaction) => { /* ... */ }
}
```

## Program Helpers

- **System** (`instructions::system`): `create_account`, `create_account_with_seed`, `assign`, `assign_with_seed`, `transfer`, `transfer_with_seed`, `allocate`, `allocate_with_seed`, `create_nonce_account`, `create_nonce_account_with_seed`, `initialize_nonce_account`, `advance_nonce_account`, `withdraw_nonce_account`, `authorize_nonce_account`, `upgrade_nonce_account`
- **SPL Token** (`instructions::token`, each with a `*_with_program_id` variant for Token-2022): `initialize_mint`, `initialize_account`, `transfer`, `transfer_checked`, `mint_to`, `mint_to_checked`, `burn`, `burn_checked`, `close_account`, `sync_native`
- **Associated Token Account** (`instructions::associated_token`): `create_associated_token_account`, `create_associated_token_account_idempotent`, `get_associated_token_address`
- **Compute Budget** (`instructions::compute_budget`, legacy/v0): `set_compute_unit_limit`, `set_compute_unit_price`, `request_heap_frame`, `set_loaded_accounts_data_size_limit`, `ensure_compute_unit_price`
- **Memo** (`instructions::memo`): `memo`
- **Anchor** (`instructions::anchor`): `global_discriminator`, `account_discriminator`, `event_discriminator`

Program and sysvar addresses are constants in `instructions::program_ids` (for example `SYSTEM_PROGRAM`, `TOKEN_PROGRAM`, `COMPUTE_BUDGET_PROGRAM`), with matching base58 `*_ID` strings and helper functions.

## Examples

See `solana-primitives/examples/`:

- `basic` - build, sign, and serialize a legacy transaction
- `decode_tx` - decode and inspect a mainnet transaction

```bash
cargo run --example basic
cargo run --example decode_tx
```

## License

[Apache-2.0](LICENSE)
