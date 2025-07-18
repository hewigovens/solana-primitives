# Solana Primitives

[![Rust](https://github.com/hewigovens/solana-primitives/actions/workflows/rust.yml/badge.svg)](https://github.com/hewigovens/solana-primitives/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/solana-primitives.svg)](https://crates.io/crates/solana-primitives)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hewigovens/solana-primitives)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This crate provides the fundamental data structures and tools needed to construct and submit Solana transactions.

## Features

- Core Solana data structures (Pubkey, Signature, Instruction, Transaction)
- Transaction and Instruction builders
- RPC client for interacting with Solana nodes

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
solana-primitives = "0.1.0"
```

### Basic Example

```rust
use solana_primitives::{
    Pubkey, InstructionBuilder, TransactionBuilder, RpcClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an RPC client
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    // Get a recent blockhash
    let (recent_blockhash, _) = rpc_client.get_latest_blockhash().await?;

    // Create a fee payer (using a test keypair)
    let fee_payer = Pubkey::from_base58("11111111111111111111111111111111")?;

    // Create an instruction using the System Program
    let program_id = Pubkey::from_base58("11111111111111111111111111111111")?; // System Program ID
    let instruction = InstructionBuilder::new(program_id)
        .account(fee_payer, true, true)
        .data(vec![2, 1, 0, 0, 0, 0, 0, 0, 0]) // Transfer instruction with 1 lamport
        .build();

    // Create and build a transaction
    let mut transaction_builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    transaction_builder.add_instruction(instruction);
    let transaction = transaction_builder.build()?;

    // Serialize the transaction
    let transaction_bytes = borsh::to_vec(&transaction)?;

    // Submit the transaction
    let signature = rpc_client.submit_transaction(&transaction_bytes).await?;
    println!("Transaction submitted: {}", signature);

    Ok(())
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
