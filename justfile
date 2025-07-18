lint:
    cargo clippy

lint-fix:
    cargo clippy --fix --allow-dirty --allow-staged

test:
    cargo test

build:
    cargo build

example-basic:
    cd examples/basic && cargo run

example-decode-tx:
    cd examples/decode_tx && cargo run
