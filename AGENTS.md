# AGENTS.md

This file provides guidance to Coding Agents (Claude Code, OpenAI Codex, Google Gemini) when working with code in this repository.

## Common Commands

### Building and Testing
- `cargo build` - Build the workspace
- `cargo test` - Run all tests
- `just lint-fix` - Run linter with auto-fixes
- `just lint` - Run clippy via justfile
- `just test` - Run tests via justfile
- `just build` - Build via justfile

### Examples
- `just example-basic` - Run the basic example
- `just example-decode-tx` - Run the decode transaction example
- `cargo run --example basic` - Run basic example via Cargo examples
- `cargo run --example decode_tx` - Run decode transaction example via Cargo examples

## Architecture Overview

This crate provides fundamental Solana blockchain primitives for constructing and submitting transactions without requiring the full Solana SDK.

### Core Module Structure

- **`types/`** - Core Solana data structures (Pubkey, Transaction, Instruction, Message, etc.)
- **`builder.rs`** - High-level builders for transactions and instructions (`TransactionBuilder`, `InstructionBuilder`)
- **`instructions/`** - Pre-built instruction constructors for common Solana programs (System, Token, etc.)
- **`crypto/`** - Cryptographic utilities and key handling
- **`borsh_helpers.rs`** - Serialization utilities for Borsh format
- **`short_vec.rs`** - Compact vector encoding utilities

### Key Design Patterns

1. **Builder Pattern**: `TransactionBuilder` and `InstructionBuilder` provide fluent APIs for constructing transactions
2. **Minimal Dependencies**: Core functionality with minimal external dependencies to keep the crate lightweight
3. **Account Metadata Management**: `TransactionBuilder` automatically manages account metadata and deduplication
4. **Workspace Structure**: Main library in `solana-primitives/` with examples under `solana-primitives/examples/` using Cargo examples

### Transaction Construction Flow

1. Create `TransactionBuilder` with fee payer and recent blockhash
2. Build individual instructions using `InstructionBuilder` or pre-built instruction modules
3. Add instructions to transaction builder via `add_instruction()`
4. Call `build()` to compile into final `Transaction` with proper account ordering and metadata

### Testing Strategy

- Unit tests are co-located with implementation files
- Examples serve as integration tests demonstrating real usage patterns
- CI runs on push/PR to main branch with build and test validation

## Coding Guidelines

### Import and Export Rules

1. **Don't re-export common types in modules**:
   - ❌ **BAD**: Adding re-exports like `pub use crate::types::{AccountMeta, Instruction, Pubkey};` in individual modules
   - ✅ **GOOD**: Keep type exports centralized in `lib.rs` or appropriate parent modules
   - **Reason**: Prevents import conflicts and maintains clear module boundaries

2. **Never add use statements inside function bodies**:
   - ❌ **BAD**:
     ```rust
     fn my_function() {
         use crate::types::MAX_TRANSACTION_SIZE;
         // function code
     }
     ```
   - ✅ **GOOD**:
     ```rust
     use crate::types::MAX_TRANSACTION_SIZE;

     fn my_function() {
         // function code
     }
     ```
   - **Reason**: Keeps imports at the top of the file for better readability and maintainability

### Code Quality

3. **Always lint and format before committing code**:
   - **Required**: Run `just lint-fix` to fix lint issues
   - **Required**: Run `cargo fmt` to format code consistently
   - **Required**: Ensure all tests pass with `cargo test`
   - **Best Practice**: Use `just lint` and `just test` shortcuts when available
   - **Reason**: Maintains consistent code quality and prevents CI failures
