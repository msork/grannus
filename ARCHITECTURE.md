# Architecture

## System context

```text
Switch 2 HDMI -> capture card -> Grannus -> encrypted network -> Sirona
                                      ^                         |
                                      | normalized input        |
                                      +-------------------------+
                                      |
                         Bluetooth/bridge controller reports
                                      |
                                  Switch 2
```

The media and input loops are intentionally asymmetric. Media is a paced stream
that may recover selectively from loss. Input is a small, high-rate state
snapshot where freshness dominates completeness.

## Process model

Initially Grannus is one host process with isolated tasks and bounded queues.
Split processes only when fault containment, privilege separation, or vendor
SDK constraints justify the extra IPC and latency.

Logical components:

| Component | Responsibility | Must not own |
| --- | --- | --- |
| Session manager | pairing, auth, capabilities, leases, lifecycle | codec/device details |
| Capture backend | frames/audio + timestamps/capabilities | network sessions |
| Encoder backend | negotiate and encode low-latency media | controller state |
| Media sender | packetization, pacing, loss strategy | capture APIs |
| Input receiver | authenticate, parse, reject stale input | Bluetooth details |
| Slot manager | authorize identity-to-player/controller lease | wire codec |
| Controller backend | convert normalized state to console reports | client brands |
| Metrics/trace | bounded observations and correlated timestamps | secrets/payload logs |

## Crate plan

The initial workspace is deliberately small. Add backend crates only when their
first tested implementation begins.

```text
grannus-core       domain types, state machines, errors, capabilities
grannus-protocol   versioned wire messages and codecs
grannus-platform   backend traits, timestamped frames, fake backends
grannus-host       config, lifecycle, CLI/service composition

future:
grannus-capture-v4l2
grannus-encode-vaapi / nvenc / vulkan-video
grannus-audio-alsa / pipewire
grannus-transport-*
grannus-controller-bluez-switch1
grannus-discovery-mdns
grannus-nat
```

Backend feature flags must not create different protocol semantics. Capability
negotiation describes actual runtime support.

## Data path

### Media

1. Capture backend negotiates an explicit format and device buffer model.
2. Each frame receives a monotonic capture-arrival timestamp and opaque handle.
3. Conversion is avoided when encoder import supports the capture buffer.
4. Encoder runs a low-delay configuration and records start/end timestamps.
5. A bounded queue drops according to documented media policy instead of
   accumulating latency.
6. Sender packetizes/paces without coupling media loss to control reliability.
7. Sirona feeds decode/present timing back into session telemetry.

### Input

1. Sirona samples a normalized full controller state and assigns a wrapping
   sequence number plus monotonic client timestamp.
2. Grannus authenticates the datagram before parsing/dispatch.
3. Parser enforces fixed maximum sizes and known protocol versions.
4. Slot manager verifies client identity and active lease.
5. Sequence window rejects duplicates/reordered stale state.
6. Controller backend emits the newest usable state at the device report cadence.
7. Haptic output travels on a separate return path with bounded lifetime.

## Queue policy

Every queue declares capacity and overflow behavior. Defaults:

| Boundary | Capacity concept | Overflow |
| --- | --- | --- |
| capture -> encode | 1–2 frames | drop oldest/not-yet-encoded frame |
| encode -> packetize | codec-dependent small bound | request/retain recoverable point |
| input receive -> slot | one latest snapshot per source | replace older snapshot |
| slot -> controller | one latest snapshot per virtual controller | replace older snapshot |
| control operations | bounded command queue | reject/timeout with error |

Exact values are measured and configurable within safe limits.

## Time model

Each process uses a monotonic clock. Network messages carry timestamps for
relative deltas, not as globally synchronized truth. Clock-offset estimation is
versioned separately. Traces correlate `session_id`, non-secret `stream_id`,
frame/input sequence, and stage timestamps. Logs never include pairing secrets.

## Portability

Platform APIs terminate at traits. Buffer handles are capability-tagged opaque
objects so Linux DMA-BUF can be zero-copy without forcing that model on
AVFoundation or Media Foundation. Controller backends advertise emulated model,
motion/haptic features, adapter needs, and evidence status.

## Failure behavior

- Losing a client releases or grace-periods its slot according to lease policy.
- Losing capture pauses media without accepting unauthorized replacement input.
- Encoder failure attempts a bounded restart, then degrades explicitly.
- Bluetooth failure neutralizes controller state before reconnect attempts.
- Network congestion drops stale media/input; memory use remains bounded.
- No automatic Internet exposure or insecure fallback.

## Decisions still requiring prototypes

- Media/session transport: tuned RTP/UDP, QUIC datagrams, WebRTC/ICE, or a
  GameStream-style design.
- Linux capture/encode zero-copy combinations per hardware vendor.
- BlueZ peripheral/profile control that works reliably across adapters.
- Multiple emulated controllers per adapter vs one adapter per identity.
- Dedicated MCU/USB-gadget bridge protocol and hardware.

Record decisions using `docs/adr/0000-template.md`.
