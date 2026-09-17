# AGENTS.md

This file provides guidance to Coding Agents (Claude Code, OpenAI Codex, Google Gemini) when working with code in this repository.

## Common Commands

### Building and Testing
- `just test` - Run tests with default, no default, and all features
- `just lint` / `just lint-fix` - Run clippy, with or without auto-fixes
- `just fmt-check` - Check formatting
- `just doc` - Build rustdoc with warnings denied
- `just example-basic` / `just example-decode-tx` - Run an example
- `just list` - Show every recipe

## Architecture Overview

This crate provides Solana transaction primitives (legacy, v0, and v1/SIMD-0385) without requiring the full Solana SDK. The core depends only on `bs58`, `sha2`, and `curve25519-dalek` (PDA curve checks); the default `signing` feature adds `ed25519-dalek`. `serde` and `borsh` are opt-in features for the Rust data model and are never the wire format.

### Core Module Structure

- **`types/`** - Data model: `Pubkey`, `SignatureBytes`, `Instruction`/`AccountMeta`/`CompiledInstruction`, `Message` (legacy), `MessageV0`, `VersionedMessage`, `VersionedTransaction { signatures, message }` (the only transaction type), lookup tables, PDAs
- **`types/v1.rs`** - Transaction v1: `MessageV1`, `TransactionConfig`, `TransactionConfigMask`, v1 limits
- **`wire.rs`** (internal) - The only wire codec: `WireReader`, append-only writers, legacy/v0 bodies, the v1 fixed layout, and both transaction envelopes
- **`compiler.rs`** (internal) - `CompiledKeys`: one account compiler for legacy/v0/v1 (role merging, Solana SDK key order, v0 lookup extraction that keeps signers, programs, and the durable nonce static)
- **`builder/`** - `TransactionBuilder` (`build`, `build_v0`, `build_v1`), `InstructionBuilder`, `InstructionDataBuilder`
- **`instructions/`** - System, SPL Token, ATA, Compute Budget, Memo, Anchor helpers; `program_ids` holds const `Pubkey`s
- **`crypto/`** - SHA-256; key derivation, signing, and verification behind the `signing` feature
- **`short_vec.rs`** (internal) - Canonical compact-u16 length encoding used by the wire codec
- **`error.rs`** - `SolanaError` with typed `CompileError`/`DecodeError`/`EncodeError`/`SanitizeError`

### Key Design Patterns

1. **One codec, one compiler**: every version goes through `wire` and `compiler`; don't add per-version copies of encoding or key ordering
2. **Parse = decode + sanitize**: `deserialize` rejects malformed bytes, size-limit violations, and anything `sanitize()` rejects; `sanitize()` is public for in-memory values
3. **Version-aware fees**: legacy/v0 read Compute Budget instructions (micro-lamports per CU); v1 reads `TransactionConfig` (total lamports). Don't mix the two
4. **No post-compilation mutation**: add instructions to `TransactionBuilder` and recompile; only invariant-preserving setters exist on transactions
5. **Workspace Structure**: Main library in `solana-primitives/` with examples under `solana-primitives/examples/`

### Transaction Construction Flow

1. Create `TransactionBuilder` with fee payer and recent blockhash
2. Build instructions with `InstructionBuilder` or the `instructions` modules
3. Add them with `add_instruction()` / `add_instructions()`
4. Call `build()`, `build_v0(&tables)`, or `build_v1(config)`; the result is sanitized and has placeholder signatures
5. `sign()` / `partial_sign()` (`signing` feature), or sign `serialize_message()` elsewhere and `add_signature()`; then `serialize()`

### Testing Strategy

- Unit tests are co-located with implementation files; shared fixtures live in `src/test_utils/`
- Wire-format and instruction bytes are checked against golden vectors generated with the Solana SDK (`src/test_utils/vectors.rs`, built from `src/test_utils/scenarios.rs` with keys `sha256(label)`). Self-roundtrips alone are not enough
- Use `hexlit::hex!` for opaque byte fixtures
- Examples are compiled by `cargo test` and CI but never run, so they are not assertions
- Tests must pass with default features, `--no-default-features`, and `--all-features` (`just test`); gate signing-only tests with `cfg(feature = "signing")`
- CI runs build and tests across a feature matrix, an MSRV check, clippy, rustfmt, and rustdoc on push/PR to main

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
   - **Required**: Ensure all tests pass with `just test` (default and all features)
   - **Best Practice**: Use `just lint` and `just test` shortcuts when available
   - **Reason**: Maintains consistent code quality and prevents CI failures

## Version Control: Jujutsu (jj)

This repo uses [Jujutsu](https://jj-vcs.github.io/jj/) as the primary VCS, co-located with Git (the `.git` directory still exists, so `git` commands also work — but prefer `jj`).

### Common commands

- `jj st` — show working-copy status
- `jj log` / `jj log -r '..@'` — show recent changes
- `jj diff` — show working-copy diff
- `jj new` — start a new change on top of `@`
- `jj describe -m "..."` — set or amend the current change's message
- `jj commit -m "..."` — describe the current change and start a new empty one
- `jj squash` — fold the current change into its parent
- `jj git push` — push changes to the Git remote
- `jj git fetch` — pull updates from the Git remote

### Workflow notes

- jj has no staging area: every working-copy edit is part of the current change automatically.
- Amending is cheap and non-destructive — `jj describe`/`jj squash` rewrite the current change, and jj keeps the old version in the op log (`jj op log` / `jj op restore` to undo).
- For PRs, push the change with `jj git push -c @` (creates a Git branch from the current change) or push an existing branch with `jj git push --branch <name>`.

## JayJay Review Notes

The user reviews diffs with the [JayJay](http://jayjay.hewig.dev) macOS app and can leave inline notes anchored to lines of the current `jj` working-copy diff. When the user mentions review notes or asks you to check JayJay, use the `jayjay` CLI:

- `jayjay review notes` — list open notes for the current working-copy change. Each entry shows the `file:line` anchor, a snippet of the anchored line, a note id, and the note body.
- `jayjay review resolve-note <ID>` — mark a note resolved by its id. Only resolve a note after actually addressing the change it asks for, and only when the user asked you to resolve it (or resolving is the obvious next step right after fixing it in the same turn) — it's a write to an external tool, not something to do proactively.
- `jayjay review add-note --file <FILE> --line <LINE> -m <MESSAGE>` — add a note anchored to a diff line (`--side old|new` picks which side the line number refers to, default `new`). Rarely needed by an agent; mainly for the human reviewer.
