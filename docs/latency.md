# Latency measurement plan

Grannus must report stage timing rather than one unqualified latency number.

## Media stages

Capture signal/device buffering, host buffer arrival, conversion/import, encode
queue/start/end, packetization/queue/send, network arrival, client decode
queue/start/end, and presentation. When device timestamps differ from the host
clock, record correlation method and error bounds.

## Input stages

Client sampling, client send, host receive/auth/parse, slot acceptance, backend
report construction/send, Bluetooth/bridge delivery when observable, and console
response through a high-speed-camera or loopback method.

## Reporting

Report p50/p95/p99/max, sample count, warm-up, drops, stale work, and queue depth.
State host/client hardware, OS/kernel, capture card/firmware/mode, GPU/driver,
codec settings, display mode, network topology, controller adapter, and build
commit. An average alone is not an acceptable result.

## Baseline experiments

1. Capture card passthrough vs captured-frame arrival using a millisecond timer.
2. Capture-to-encoded-packet with synthetic frames and then real frames.
3. Network loopback, clean LAN, and controlled loss/reorder/jitter.
4. Sirona packet-to-present timing per decoder/render backend.
5. Input loopback to fake backend, then real Bluetooth/bridge to on-screen result.

Never compare Grannus to Moonlight/Artemis/Sunshine/Apollo/Vibepollo without
matching resolution, refresh rate, codec, network, display, and measurement
method as closely as possible.
