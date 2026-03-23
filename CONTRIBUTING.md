# Contributing to solana-primitives

## Requirements

- Rust stable toolchain (2024 edition)
- `just` (optional)

## Development loop

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Or use `just`:

```bash
just lint-fix
just lint
just test
just build
```

## Running examples

```bash
cargo run --example basic
cargo run --example decode_tx
```
