list:
    just --list

format:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --all-targets -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings
    cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
    cargo clippy --fix --all-features --all-targets --allow-dirty --allow-staged

test:
    cargo test --workspace
    cargo test --workspace --no-default-features
    cargo test --workspace --all-features

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

example-basic:
    cargo run --example basic

example-decode-tx:
    cargo run --example decode_tx
