# Grannus

Grannus is the Linux-first host for a low-latency Nintendo Switch 2 remote-play
system. A capture card supplies console video and audio; Grannus encodes and
streams it to authenticated **Sirona** clients, receives normalized controller
input, and presents that input to the console through Nintendo-compatible
controller backends.

> [!IMPORTANT]
> This repository is pre-alpha. Switch 1 Pro Controller/Joy-Con emulation is the
> MVP research direction. Switch 2-native controller impersonation is
> experimental and is not claimed as working until reproduced on real hardware.

## Current status

The repository contains a compilable, dependency-light Rust foundation:

- normalized controller types and validation;
- a bounded, versioned controller-input datagram codec;
- platform backend traits and deterministic fake backends;
- a host CLI that exercises the fake path;
- architecture, threat model, protocol, research, and milestone documents;
- CI, issue templates, contribution rules, and packaging scaffolding.

It does **not** yet stream real capture-card media or impersonate a controller.
See [ROADMAP.md](ROADMAP.md) and [CODEX_START.md](CODEX_START.md).

## Quick start

Requires the stable Rust toolchain.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -p grannus-host -- doctor
cargo run -p grannus-host -- fake-session
```

For repeatable commands, install `just` and run `just check`.

## Repository layout

| Path | Purpose |
| --- | --- |
| `crates/grannus-core` | Platform-neutral domain types and invariants |
| `crates/grannus-protocol` | Versioned wire formats and bounded codecs |
| `crates/grannus-platform` | Capture, transport, and controller interfaces |
| `crates/grannus-host` | Host process and CLI composition root |
| `docs/` | Architecture details, protocol, security, ADRs, and research |
| `config/` | Safe example configuration |
| `packaging/` | Service/install integration |

Sirona is a separate client project. Shared protocol compatibility is governed
by `docs/protocol.md`, not by importing host internals.

## Scope and support policy

Platform priority is Linux x86_64, Linux ARM64, macOS x86_64/ARM64, Windows
x86_64, then Windows ARM64 if required APIs are viable. Linux is the reference.

The project will prefer Rust and maintained permissively licensed dependencies.
Code from GPL projects must not be copied unless the repository deliberately
changes to a GPL-compatible licensing strategy.

## Legal

Grannus is an independent interoperability project and is not affiliated with
or endorsed by Nintendo. Nintendo Switch, Joy-Con, and related names are
trademarks of their respective owners. Only use capture and remote play in ways
allowed by applicable law, game rules, and service terms.

Licensed under the MIT License. See [LICENSE](LICENSE).
