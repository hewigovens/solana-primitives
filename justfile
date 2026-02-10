list:
    just --list

format:
    cargo fmt --all

lint:
    cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
    cargo clippy --fix --all-features --all-targets --allow-dirty --allow-staged

test:
    cargo test --workspace --all-features

build:
    cargo build

example-basic:
    cargo run --example basic

example-decode-tx:
    cargo run --example decode_tx
