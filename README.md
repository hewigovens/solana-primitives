# Solana Primitives

[![CI](https://github.com/hewigovens/solana-primitives/actions/workflows/ci.yml/badge.svg)](https://github.com/hewigovens/solana-primitives/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/solana-primitives.svg)](https://crates.io/crates/solana-primitives)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/solana-primitives)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight Rust crate providing fundamental Solana blockchain primitives for constructing and submitting transactions without requiring the full Solana SDK.

## Features

- **Core Solana Types**: `Pubkey`, `Signature`, `Instruction`, `Transaction`, `Message`
- **Builder Pattern APIs**: Fluent `TransactionBuilder` and `InstructionBuilder` with automatic account management
- **Instruction Data Builder**: Type-safe `InstructionDataBuilder` for constructing instruction payloads
- **Program Helpers**: Pre-built instructions for System, Token, and other common programs
- **Program ID Utilities**: Helper functions for common program IDs (System, Token, Token 2022, etc.)
- **PDA Support**: Program Derived Address generation and validation
- **Error Handling**: Comprehensive error types with detailed context messages
- **Lightweight**: Minimal dependencies for reduced bloat

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
solana-primitives = "0.2.0"
```

### Quick Start

```rust
use solana_primitives::{
    Pubkey, InstructionBuilder, InstructionDataBuilder, TransactionBuilder,
    instructions::program_ids::system_program,
    instructions::system::transfer,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fee_payer = Pubkey::from_base58("11111111111111111111111111111111")?;
    let recipient = Pubkey::from_base58("22222222222222222222222222222222")?;
    let recent_blockhash = [0u8; 32]; // In practice, get this from RPC

    // Method 1: Using pre-built instruction helpers
    let transfer_instruction = transfer(&fee_payer, &recipient, 1_000_000); // 0.001 SOL

    // Method 2: Using InstructionBuilder with InstructionDataBuilder
    let custom_instruction = InstructionBuilder::new(system_program())
        .account(fee_payer, true, true)  // signer + writable
        .account(recipient, false, true) // writable
        .data(
            InstructionDataBuilder::new()
                .instruction(2)     // Transfer instruction discriminant
                .u64(1_000_000)     // Amount in lamports
                .build()
        )
        .build();

    // Build transaction
    let mut tx_builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    tx_builder.add_instruction(transfer_instruction);

    let transaction = tx_builder.build()?;
    println!("Transaction created with {} instructions", transaction.message.instructions.len());

    Ok(())
}
```

### Advanced Usage

#### Program Derived Addresses (PDAs)

```rust
use solana_primitives::{find_program_address, Pubkey};

let program_id = Pubkey::from_base58("YourProgramId11111111111111111111111111")?;
let seeds = [b"your_seed", b"another_seed"];
let seed_refs: Vec<&[u8]> = seeds.iter().map(|s| s.as_ref()).collect();

let (pda, bump) = find_program_address(&program_id, &seed_refs)?;
println!("PDA: {}, Bump: {}", pda.to_base58(), bump);
```


### Error Handling

The crate provides detailed error context:

```rust
use solana_primitives::{Pubkey, Result, SolanaError};

fn example() -> Result<()> {
    // This will provide a detailed error message
    let invalid_pubkey = Pubkey::from_base58("invalid")?;
    Ok(())
}

// Error message: "Invalid public key: failed to decode base58: invalid"
```

## Available Program Helpers

The crate includes pre-built instruction constructors for common Solana programs:

- **System Program**: `transfer`, `create_account`, `allocate`, etc.
- **Token Program**: `transfer`, `transfer_checked`, `mint_to`, `burn`, etc.
- **Associated Token Program**: `create_associated_token_account`
- **Compute Budget Program**: `set_compute_unit_limit`, `set_compute_unit_price`

Program ID helpers are available for easy access:

```rust
use solana_primitives::instructions::program_ids::{
    system_program, token_program, token_2022_program,
    associated_token_program, compute_budget_program
};
```

## Examples

See `solana-primitives/examples/` for complete working examples:

- `basic` - Basic transaction construction
- `decode_tx` - Transaction deserialization

Run examples with:
```bash
cargo run --example basic
cargo run --example decode_tx
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
