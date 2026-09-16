# Grannus parser fuzzing

The `decode_input_v1` target exercises arbitrary byte slices against the v1
controller-input decoder. Every successful decode must round-trip through the
canonical encoder. Decoder failures are expected; a panic, memory-safety
failure, or failed round trip is a fuzz finding.

## Prerequisites

`cargo-fuzz` uses `libFuzzer`, which currently requires Linux, a C++ compiler,
and a Rust nightly toolchain. Install the tool separately from the production
workspace:

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
```

## Run and minimize

Run a bounded local smoke fuzz before longer campaigns:

```bash
cargo fuzz run decode-input-v1 -- -max_total_time=20
```

For a crash artifact, preserve only the minimized input after confirming it
contains no personal or secret data:

```bash
cargo fuzz tmin decode-input-v1 fuzz/artifacts/decode-input-v1/<artifact>
```

Commit a minimized regression only when it is safe to publish. Never commit raw
network captures, pairing material, keys, or identifiers. Seeds should be
generated locally with the public v1 encoder or created from sanitized synthetic
data, not copied from a client session.

## Corpus policy

`fuzz/corpus/decode-input-v1/` is populated by `cargo fuzz run`. Keep only
small, non-sensitive, reproducible seeds. Run `cargo fuzz cmin decode-input-v1`
before retaining a larger corpus.

If a sandbox blocks LeakSanitizer's required `ptrace` behavior, use
`LSAN_OPTIONS=detect_leaks=0` only for the constrained smoke run and repeat the
full leak-detection campaign on a host where LeakSanitizer can run normally.
