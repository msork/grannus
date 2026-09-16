# Development environment

## Required

- Git
- Rust stable 1.85 or newer with Rustfmt and Clippy
- a C toolchain/linker for the target

Optional tools: `just`, `cargo-deny`, a coverage tool, packet-analysis tooling,
and hardware-specific development packages. Do not require privileged hardware
access for the default workspace test suite.

## Bootstrap

```bash
rustup toolchain install stable --profile minimal --component rustfmt,clippy
cargo test --workspace --all-features
./scripts/check.sh
```

## Branch discipline

Branch from `main`, keep commits focused, and do not rewrite unrelated user
changes. Put generated/build output outside version control. Commit `Cargo.lock`
because the workspace ships an application. Review lockfile changes with every
dependency update.

## Hardware experiments

Never run a hardware experiment implicitly from `cargo test`. Mark tests that
require devices and invoke them explicitly with exact setup documentation. Add
results to `docs/hardware-matrix.md`; preserve raw sensitive captures outside
Git and commit only sanitized, minimal derived evidence when appropriate.

## Logging

Logs should be machine-readable once a logging dependency is selected. Avoid
payloads, tokens, pairing codes/keys, MAC/IP addresses by default, and unbounded
per-packet output. Session/stream correlation IDs must be non-secret.
