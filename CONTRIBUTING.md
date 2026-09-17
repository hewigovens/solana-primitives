# Contributing to solana-primitives

## Requirements

- Rust 1.88 or newer (2024 edition)
- `just` (optional)

## Development loop

```bash
just format      # cargo fmt --all
just lint        # clippy with default, no-default and all features
just test        # tests with default, no-default and all features
just doc         # rustdoc with warnings denied
```

`just list` shows every recipe, including `example-basic` and `example-decode-tx`.
CI runs the same checks, so a green `just lint && just test` is a good predictor.

## Releasing

See [docs/RELEASING.md](docs/RELEASING.md) for cutting a new release.
