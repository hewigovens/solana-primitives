# AGENTS.md

This file provides guidance to Coding Agents (Claude Code, OpenAI Codex, Google Gemini_ when working with code in this repository.

## Common Commands

### Building and Testing
- `cargo build` - Build the workspace
- `cargo test` - Run all tests
- `cargo clippy --fix --allow-dirty --allow-staged` - Run linter with auto-fixes
- `just lint` - Run clippy via justfile
- `just test` - Run tests via justfile
- `just build` - Build via justfile

### Examples
- `just example-basic` - Run the basic example
- `just example-decode-tx` - Run the decode transaction example
- `cd examples/basic && cargo run` - Run basic example directly
- `cd examples/decode_tx && cargo run` - Run decode transaction example directly

## Architecture Overview

This crate provides fundamental Solana blockchain primitives for constructing and submitting transactions without requiring the full Solana SDK.

### Core Module Structure

- **`types/`** - Core Solana data structures (Pubkey, Transaction, Instruction, Message, etc.)
- **`builder.rs`** - High-level builders for transactions and instructions (`TransactionBuilder`, `InstructionBuilder`)
- **`instructions/`** - Pre-built instruction constructors for common Solana programs (System, Token, etc.)
- **`jsonrpc/`** - Optional RPC client for interacting with Solana nodes (behind `jsonrpc` feature)
- **`crypto/`** - Cryptographic utilities and key handling
- **`borsh_helpers.rs`** - Serialization utilities for Borsh format
- **`short_vec.rs`** - Compact vector encoding utilities

### Key Design Patterns

1. **Builder Pattern**: `TransactionBuilder` and `InstructionBuilder` provide fluent APIs for constructing transactions
2. **Feature Gating**: RPC functionality is optional via the `jsonrpc` feature to keep core dependencies minimal
3. **Account Metadata Management**: `TransactionBuilder` automatically manages account metadata and deduplication
4. **Workspace Structure**: Main library in `solana-primitives/` with examples in separate workspace members

### Transaction Construction Flow

1. Create `TransactionBuilder` with fee payer and recent blockhash
2. Build individual instructions using `InstructionBuilder` or pre-built instruction modules
3. Add instructions to transaction builder via `add_instruction()`
4. Call `build()` to compile into final `Transaction` with proper account ordering and metadata

### Testing Strategy

- Unit tests are co-located with implementation files
- Examples serve as integration tests demonstrating real usage patterns
- CI runs on push/PR to main branch with build and test validation
