# Grannus project settings

## Repository

- Canonical repository: `https://github.com/msork/grannus`
- Default branch: `main`
- Language: Rust stable, edition 2024
- License: MIT
- MSRV: not set during pre-alpha; CI stable is authoritative
- Release posture: pre-alpha; no compatibility promises before `0.1.0`

## Product priorities

1. Correctness and controller compatibility
2. Lowest practical measured input/video latency
3. Stability and reconnect behavior
4. Security
5. Linux support
6. Portability
7. UI polish

## Platform order

1. Linux x86_64
2. Linux ARM64
3. macOS x86_64
4. macOS ARM64
5. Windows x86_64
6. Windows ARM64 if APIs are viable

Linux is the reference platform. CI portability does not imply hardware support.

## Initial technical defaults

- 48 kHz stereo audio; format remains negotiable/extensible.
- H.264 first for compatibility; evaluate HEVC and AV1 by measured path support.
- No B-frames, lookahead, or deep queues in low-latency presets unless testing
  shows a net benefit.
- LAN-first development. Internet traversal follows authenticated LAN sessions.
- Manual host/port and LAN discovery before UPnP/PCP/NAT-PMP/ICE/relay.
- Controller input uses sequenced latest-state-wins datagrams.
- Control/session operations use reliable ordered delivery.
- Media transport is selected by ADR after comparable prototypes.

These are starting hypotheses, not permanent architecture decisions.

## Quality gates

- Formatting, Clippy with warnings denied, and all workspace tests pass.
- New parsing/state-machine code has negative tests.
- Public protocol changes update `docs/protocol.md` and compatibility tests.
- Dependency additions record license, maintenance, and alternatives.
- Hardware claims include reproducible evidence and a truth label.
- Hot-path work includes before/after measurements or instrumentation.
- No secret material or GPL-derived implementation is committed.

## Performance target process

Do not invent a single end-to-end latency promise. Establish baselines per
hardware profile, then set budgets for capture, processing, encode, host queue,
network, decode, presentation, input return, and Bluetooth report delivery.
Track p50/p95/p99 and dropped/stale work.

## Dependency policy

Prefer maintained, widely used, permissively licensed crates. Pin through
`Cargo.lock` for applications. Run `cargo deny` for advisories, license, ban,
and source checks once dependencies exist. Avoid framework lock-in in core
crates. Keep codec/vendor SDKs optional and feature-gated.
