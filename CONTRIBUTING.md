# Contributing to solana-primitives

## Requirements

- Rust 1.88 or newer (2024 edition)
- `just` (optional)

## Development loop

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --no-default-features
cargo test --all-features
```

Or use `just`:

```bash
just lint-fix
just lint
just fmt-check
just test
just doc
just build
```

## Running examples

```bash
cargo run --example basic
cargo run --example decode_tx
```

## Releasing

See [docs/RELEASING.md](docs/RELEASING.md) for cutting a new release.
