
# Solana Primitives

This project provides Rust primitives for interacting with the Solana blockchain. The main crate is `solana-primitives`, which contains data structures, builders, and an RPC client for creating and sending transactions.

## Project Structure

- `solana-primitives`: The core library crate.
  - `src/`:
    - `lib.rs`: The main library file, exporting modules.
    - `builder.rs`: Contains `InstructionBuilder` and `TransactionBuilder` for constructing transactions.
    - `crypto/`: Cryptographic utilities like `Signature`.
    - `error.rs`: Defines error types for the library.
    - `instructions/`: Modules for building specific instructions (e.g., `system`, `token`).
    - `jsonrpc/`: An RPC client for communicating with a Solana node (enabled by the `jsonrpc` feature).
    - `types/`: Core Solana data structures like `Pubkey`, `Transaction`, `Instruction`, etc.
- `examples/`:
  - `basic/`: A simple example of creating and sending a transaction.
  - `decode_tx/`: An example of how to decode a base64 encoded transaction.

## How to Build, Test, and Run

The project uses a `justfile` for common commands:

- **Build:** `just build` (or `cargo build`)
- **Test:** `just test` (or `cargo test`)
- **Lint:** `just lint` (or `cargo clippy --fix --allow-dirty --allow-staged`)
- **Run `basic` example:** `just example-basic` (or `cd examples/basic && cargo run`)
- **Run `decode_tx` example:** `just example-decode-tx` (or `cd examples/decode_tx && cargo run`)

## Dependencies

The project uses `cargo` for dependency management. Key dependencies are defined in the root `Cargo.toml` and the `solana-primitives/Cargo.toml` file. The `jsonrpc` feature enables the RPC client and its dependencies (`reqwest`, `tokio`, `serde_json`).
