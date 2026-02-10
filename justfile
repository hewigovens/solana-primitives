lint:
    cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
    cargo clippy --fix --allow-dirty --allow-staged

test:
    cargo test

build:
    cargo build

example-basic:
    cargo run --example basic

example-decode-tx:
    cargo run --example decode_tx
