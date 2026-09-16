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

## Configuration dependencies

`grannus-host` uses [`serde` 1.0](https://docs.rs/serde/1.0.229/serde/) with
derive support and [`toml` 1.1](https://docs.rs/toml/1.1.6/toml/) to deserialize
local host configuration. Both are actively maintained Rust crates under MIT or
Apache-2.0 terms and have no platform-specific runtime requirements. Typed
deserialization with strict per-section schemas was selected over a handwritten
parser so unknown settings fail closed with clear TOML diagnostics. Neither
crate is used on media or input hot paths.

`grannus-core` uses [`proptest` 1.11](https://docs.rs/proptest/1.11.0/proptest/)
as a test-only dependency for generated sequence-ordering properties. It is
passively maintained by its upstream project, licensed MIT or Apache-2.0, and
declares Rust 1.85 support. It has no production runtime or platform impact.

`fuzz/` is an isolated `cargo-fuzz` 0.13.2 workspace for untrusted parser
targets. `cargo-fuzz` is actively maintained by the Rust Fuzzing Authority and
dual-licensed MIT or Apache-2.0. It uses `libfuzzer-sys` 0.4.13, whose Rust
wrapper is MIT or Apache-2.0 and whose vendored libFuzzer runtime is NCSA
licensed. The toolchain requires Linux, a C++ compiler, and Rust nightly; none
of these dependencies are shipped in the production workspace.

`grannus-host` uses [`signal-hook` 0.3.18](https://docs.rs/signal-hook/0.3.18/)
for safe Unix SIGINT/SIGTERM flags. It is dual-licensed Apache-2.0 or MIT and
is restricted to the host crate; core and platform remain signal-agnostic.
