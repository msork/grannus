# Grannus–Sirona protocol

Status: draft. This document is the interoperability authority; Rust structs are
an implementation. All multibyte integers are unsigned/signed big-endian unless
stated otherwise.

## Design rules

- Version every independently parsed message family.
- Authenticate packets before acting on decoded content.
- Bound datagrams and allocations before parsing variable data.
- Separate reliable control, time-sensitive media, and latest-state input.
- Unknown mandatory versions fail closed; negotiated optional capabilities may
  be ignored only where specified.
- The controller model is brand/OS independent.

Transport authentication, key schedule, pairing messages, and media framing are
not selected yet. Do not deploy the v1 input body alone on a network; it has no
built-in authentication and is intended to be carried inside an authenticated
session transport.

## Controller input datagram v1

A full state snapshot supersedes older accepted snapshots from the same source.
Fixed length: 48 bytes.

| Offset | Size | Field | Rule |
| ---: | ---: | --- | --- |
| 0 | 2 | magic | ASCII `GI` |
| 2 | 1 | version | `1` |
| 3 | 1 | flags | bit 0 motion valid; other bits zero |
| 4 | 4 | sequence | wrapping per input source |
| 8 | 8 | source time µs | monotonic, session-relative |
| 16 | 1 | player slot | 0–7; authorization checked separately |
| 17 | 4 | buttons | defined bits only |
| 21 | 2 | left X | signed normalized axis |
| 23 | 2 | left Y | signed normalized axis |
| 25 | 2 | right X | signed normalized axis |
| 27 | 2 | right Y | signed normalized axis |
| 29 | 2 | left trigger | unsigned normalized trigger |
| 31 | 2 | right trigger | unsigned normalized trigger |
| 33 | 6 | gyro XYZ | signed fixed-point units; negotiation TBD |
| 39 | 6 | accelerometer XYZ | signed fixed-point units; negotiation TBD |
| 45 | 3 | reserved | zero |

Button bits: south 0, east 1, west 2, north 3, D-pad up/down/left/right
4–7, shoulders 8–9, stick clicks 10–11, select/start 12–13, home 14,
capture 15, digital triggers 16–17. Bits 18–31 are zero in v1.

### Sequence ordering

M0 must define and property-test the wrapping comparison window before live
network use. Duplicate or older snapshots are dropped. After an authenticated
session restart, sequence state resets through explicit session lifecycle, not
through an unauthenticated packet.

### Staleness

Source timestamps support telemetry and negotiated freshness limits only after
clock-offset estimation exists. Host receive time is authoritative for initial
timeout behavior. On timeout/disconnect, the controller backend sends neutral
state before lease release.

## Future control plane

The reliable authenticated control plane will negotiate:

- protocol range and feature capability identifiers;
- persistent client identity and session authorization;
- codec/profile/level, resolution, refresh, bitrate, HDR/color properties;
- audio format/channels/sample rate;
- controller sources, virtual controller modes, player leases, motion/haptics;
- path/MTU and transport parameters;
- telemetry capabilities and safe diagnostic verbosity.

Normative schemas and compatibility fixtures must land before the first Sirona
integration. Generated schema formats must preserve unknown optional fields and
bound all collections.

## Compatibility policy

Before 0.1, breaking changes are allowed but must change the message version or
capability and update golden vectors. After 0.1, Grannus should support at least
the current and previous stable compatible protocol ranges. Downgrade must not
bypass security or authorization.
