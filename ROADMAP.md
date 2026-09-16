# Roadmap

Checkboxes represent merged, verified work only. Hardware items require named
hardware evidence; simulation alone cannot complete them.

## M0 — deterministic foundation

- [x] Create Rust workspace and project governance/docs.
- [x] Define normalized controller state with validation tests.
- [x] Define a bounded v1 controller-input datagram with round-trip/negative tests.
- [x] Define backend traits and deterministic fake capture/controller backends.
- [x] Add a fake-session CLI smoke path.
- [ ] Add structured configuration loading, schema validation, and safe defaults.
- [ ] Add monotonic stage timestamps, histogram-ready metric events, and trace IDs.
- [ ] Add slot lease/ownership state machine with reconnect/conflict tests.
- [ ] Add stale/duplicate/wrapping sequence-window property tests.
- [ ] Add fuzz targets for every untrusted decoder.
- [ ] Add graceful lifecycle and signal-driven shutdown.

Exit: fake capture -> fake encode/transport and synthetic input -> fake controller
run concurrently with bounded queues, metrics, fault injection, and soak tests.

## M1 — Linux capture and local media baseline

- [ ] Inventory V4L2/media capture cards and capability negotiation.
- [ ] Implement V4L2 UVC backend using memory-mapped buffers.
- [ ] Preserve device timestamps and detect discontinuities.
- [ ] Add 48 kHz stereo capture backend and A/V timestamp correlation.
- [ ] Render/record a local diagnostic stream without network transport.
- [ ] Measure capture latency with a reproducible camera/LED or timer method.
- [ ] Prototype DMA-BUF import on at least one Intel/AMD/NVIDIA path.

Exit: stable 1080p60 capture on named UVC hardware with p50/p95/p99 timing and
no unbounded growth during a two-hour soak.

## M2 — low-latency encode and LAN session

- [ ] ADR comparing transport prototypes under loss/reorder/jitter.
- [ ] Hardware H.264 low-delay backend and software fallback for testing.
- [ ] Authenticated pairing and persistent client identity.
- [ ] Reliable control channel plus separate media/input traffic classes.
- [ ] Manual IP/hostname connection and IPv4/IPv6 support.
- [ ] LAN discovery with no unauthenticated admin/input surface.
- [ ] Sirona interoperability vectors and compatibility matrix.
- [ ] End-to-end media/input telemetry.

Exit: one authorized Sirona client streams on LAN and sends input to a fake
controller with measured loss recovery and latency.

## M3 — Switch 1 controller emulation MVP

- [ ] Clean-room protocol research log and legal/license review.
- [ ] Adapter capability/identity diagnostic tool.
- [ ] Switch 1 Pro Controller pairing/reconnect/report state machine.
- [ ] Joy-Con L and Joy-Con R modes.
- [ ] Combined L+R single-player mapping.
- [ ] Separate L/R multiplayer mapping.
- [ ] Rumble/motion behavior supported by evidence.
- [ ] Neutral-state fail-safe and reconnect soak tests.
- [ ] Reproduce on Switch 2 with named adapter/kernel/firmware evidence.

Exit: real Switch 2 accepts at least one emulated original Switch controller and
plays a test title reliably. Do not infer native Switch 2 controller support.

## M4 — multi-client and resilient host

- [ ] Authorized slot assignment, admin approval, and anti-hijack rules.
- [ ] Multiple adapters and multiple controller instances.
- [ ] Client reconnect grace and deterministic conflict resolution.
- [ ] Per-session bitrate/resolution/refresh negotiation.
- [ ] Service packaging, privilege separation, diagnostics bundle.
- [ ] Crash recovery and long-duration soak tests.

## M5 — Internet traversal

- [ ] Threat-model update and abuse/rate-limit tests.
- [ ] Opt-in UPnP IGD with visible lease lifecycle.
- [ ] PCP and NAT-PMP evaluation.
- [ ] STUN/ICE-style traversal evaluation/implementation.
- [ ] Optional authenticated relay with explicit privacy/cost model.
- [ ] Adverse WAN benchmarks and safe fallback behavior.

## M6 — codecs, platforms, experimental hardware

- [ ] HEVC and AV1 capability/latency matrix.
- [ ] Linux ARM64 hardware matrix.
- [ ] macOS capture/encode feasibility and controller bridge strategy.
- [ ] Windows capture/encode feasibility and controller bridge strategy.
- [ ] Dedicated microcontroller/USB-gadget bridge prototype.
- [ ] Switch 2-native controller research, kept experimental until proven.
