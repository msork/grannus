# AGENTS.md — Codex operating contract

These instructions apply to the entire repository. More specific `AGENTS.md`
files may narrow them for a subtree but may not weaken safety, security, or
evidence requirements.

## Mission

Build Grannus as a production-minded, Linux-first host for low-latency Nintendo
Switch 2 remote play. Sirona is the separate client. Grannus captures HDMI
video/audio, encodes and streams it, receives authenticated normalized input,
and emits one or more console-facing Nintendo-compatible controllers.

Treat this as real systems software. Never turn an assumption into a support
claim. Capture-card latency is part of the budget and must be measured.

## Truth labels

Use one label in designs, issues, and PRs for hardware/protocol claims:

- **Proven**: reproduced on specified hardware with logs/traces and steps.
- **Likely viable**: supported by relevant APIs or credible prior art, not yet
  reproduced in Grannus.
- **Experimental**: implementation/research exists but compatibility varies.
- **Unproven**: no adequate evidence.

Switch 1 Pro Controller and Joy-Con Bluetooth impersonation is the MVP direction
based on public prior art. Switch 2 Pro Controller/Joy-Con 2 console-facing
impersonation remains experimental/unproven until a computer is demonstrated
impersonating one to a Switch 2. A real controller connecting to a PC is not
equivalent evidence.

## Non-negotiable engineering rules

1. Read `PROJECT_SETTINGS.md`, `ARCHITECTURE.md`, `ROADMAP.md`, relevant docs,
   and current code before changing architecture.
2. Work on the first unchecked roadmap item whose prerequisites are satisfied,
   unless the user names another task.
3. Keep core logic platform-independent. OS/device code lives behind small
   traits in `grannus-platform` or a backend crate.
4. Use Rust wherever practical. Native shims require a written justification,
   a narrow safe interface, and tests around the boundary.
5. No custom cryptography. Use established audited libraries and protocols.
6. Do not expose unauthenticated input, pairing, metrics, or admin endpoints.
7. All untrusted lengths, counts, enums, timestamps, and identifiers are
   validated before allocation or use. Fuzz parsers when introduced.
8. High-rate input is latest-state-wins. Never let reliable retransmission
   deliver stale input or block fresh media/input.
9. Avoid hot-path copies, allocations, locks, buffering, and thread hops, but
   optimize only after instrumentation identifies a bottleneck.
10. No `unsafe` by default. If unavoidable, add a `// SAFETY:` invariant,
    isolate it, test it, and explain it in the PR.
11. Never copy GPL code into this MIT repository. Protocol facts may be
    independently reimplemented from documented observations. Record source,
    license, and clean-room notes in `docs/research/`.
12. Never commit secrets, pairing keys, packet captures containing personal
    identifiers, proprietary SDKs, firmware, ROMs, or copyrighted keys.

## Required workflow

Before coding:

1. Restate the chosen milestone and acceptance criteria.
2. Inspect the tree and related tests.
3. Research current APIs/dependencies if a choice may have changed; prefer
   primary docs/repos and record links, versions, maintenance, license, and
   platform constraints.
4. Write or update an ADR before a hard-to-reverse dependency/protocol choice.

While coding:

- Make the smallest end-to-end change that can be verified.
- Preserve public API compatibility or version it explicitly.
- Prefer bounded channels and explicit backpressure/drop policy.
- Use monotonic time internally; wall-clock time is diagnostic metadata only.
- Keep device I/O out of async executor blocking threads.
- Add metrics at stage boundaries, not per-packet high-cardinality labels.
- Add unit tests for invariants and integration tests for behavior.

Before finishing:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Also run task-specific tests. If a tool/hardware dependency prevents a check,
say exactly what was not run and why. Do not claim success from inspection.

## Architecture boundaries

- `grannus-core`: domain types only; no OS or network stack dependencies.
- `grannus-protocol`: explicit versioned wire representations; no device I/O.
- `grannus-platform`: traits, capabilities, and fake/reference backends.
- `grannus-host`: configuration, lifecycle, orchestration, CLI/service shell.
- Future backend crates: one concern each (`capture-v4l2`, `encode-vaapi`,
  `controller-bluez-switch1`, transports, discovery, NAT traversal).

Do not let V4L2, BlueZ, XInput, SDL, evdev, or a specific codec leak into the
normalized controller/session protocol.

## Latency and reliability acceptance

Every media stage must expose timestamps/metrics for capture arrival, encode
start/end, queue wait, send, client receipt/decode/present where available.
Input must expose client sample time, host receive time, accepted sequence, drop
reason, and controller report time. Report distributions (p50/p95/p99), not only
averages. Benchmarks must state hardware, kernel/OS, capture mode, codec,
resolution, refresh rate, network, and sample count.

Tests must cover loss, reorder, duplication, stale sequences, reconnect,
malformed packets, capability negotiation, and bounded resource behavior.

## Controller rules

- Model buttons, sticks, analog triggers, gyro, accelerometer, connection,
  player assignment, and haptics independently of brands.
- Clamp/validate axes at trust boundaries; preserve timestamps and sequence.
- Slot ownership is explicit and authorized. Reconnect may reclaim only the
  identity's authorized lease. No silent slot hijacking.
- Treat each Bluetooth adapter's identity and concurrency as a resource.
  Multiple virtual controllers per adapter is not promised until tested.
- Support multiple USB adapters and investigate microcontroller/USB-gadget
  bridges when desktop Bluetooth cannot provide required peripheral behavior.

## Network and security rules

Keep three traffic classes logically separate:

1. reliable session/control;
2. unreliable/time-sensitive media;
3. unreliable/latest-state controller input.

Pair through explicit user confirmation/code, create a persistent client
identity, authenticate every session, and store secrets with restrictive
permissions or OS secure storage. Rate-limit pairing/admin operations. Bind
locally by default during early development. UPnP/PCP/NAT-PMP are opt-in and
must never create an unauthenticated path.

## Commit and PR style

Use focused conventional commits such as `feat(protocol): ...`, `fix(input):
...`, `docs(adr): ...`, or `test(controller): ...`. PRs include: scope,
truth-label changes, security impact, latency impact, tests, hardware matrix,
and remaining risks. Do not mix formatting-only churn with behavior changes.

## Stop and ask

Stop for user direction when a choice would materially change licensing,
protocol interoperability, supported hardware cost, privacy exposure, or
repository ownership; when required hardware evidence is unavailable; or when
credentials/authority are needed to publish externally.
