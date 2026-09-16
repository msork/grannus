set shell := ["bash", "-euo", "pipefail", "-c"]

default: check

fmt:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

check: fmt lint test

doctor:
    cargo run -p grannus-host -- doctor

fake-session:
    cargo run -p grannus-host -- fake-session
