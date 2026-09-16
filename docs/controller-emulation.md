# Controller emulation strategy

## Support classification

| Mode | Initial status | Completion evidence |
| --- | --- | --- |
| Switch 1 Pro Controller to Switch 2 | likely viable MVP | Switch 2 pairing, reconnect, report and soak traces |
| Switch 1 Joy-Con L/R | likely viable MVP | same, including separate/combined assignments |
| Multiple virtual controllers per adapter | unproven | adapter-specific concurrent soak evidence |
| Switch 2 Pro Controller impersonation | experimental/unproven | computer/bridge impersonates it to Switch 2 |
| Joy-Con 2 L/R impersonation | experimental/unproven | same; real controller-to-PC is insufficient |

Public projects such as joycontrol and NXBT establish useful prior art for
original Switch controller emulation on Linux, but implementation must be clean
room and license-aware. `joycontrol` is GPL-3.0; do not copy it into this MIT
repository. NXBT is a useful independent reference whose current license and
specific borrowed material must still be reviewed before reuse.

## Backend contract

A controller backend advertises emulated model, supported reports/features,
required adapter identity, pairing/reconnect status, report cadence, and an
evidence label. It accepts normalized complete snapshots, not SDL/XInput/evdev
events. It provides bounded haptic events separately.

## Linux research sequence

1. Capture sanitized SDP/HID/report behavior from hardware the user owns.
2. Document public protocol facts, sources, licenses, and uncertainty.
3. Build report encoders/decoders from documented fields and golden vectors.
4. Test state machine against a fake L2CAP peer.
5. Implement the smallest BlueZ/raw Bluetooth boundary with least privileges.
6. Pair through Switch 2 Change Grip/Order and record exact evidence.
7. Test reconnect, neutralization, radio interference, sleep/wake, and soak.
8. Repeat for each adapter/kernel/firmware and multiple-adapter topology.

## Alternative bridge

If desktop Bluetooth stacks cannot reliably act as the required peripheral,
use one dedicated MCU/Bluetooth peripheral or Linux USB-gadget-class bridge per
controller identity. Compare latency, availability, firmware maintainability,
USB/radio isolation, and cost. The bridge protocol must authenticate privileged
commands where the physical link is not inherently trusted and must neutralize
on heartbeat loss.
