# Hardware evidence matrix

This file begins empty by design. Add a row only after a reproducible test, and
link the sanitized evidence. Passing CI is not hardware evidence.

## Capture

| Device/firmware | Host/kernel | Mode | Backend/buffer path | Result | Timing evidence | Truth label |
| --- | --- | --- | --- | --- | --- | --- |
| _none tested_ | | | | | | Unproven |

## Encode

| GPU/driver | Codec/settings | Import path | Resolution/rate | Soak | Timing evidence | Truth label |
| --- | --- | --- | --- | --- | --- | --- |
| _none tested_ | | | | | | Unproven |

## Console-facing controllers

| Console/firmware | Virtual model | Adapter/firmware | Host/kernel | Pair/reconnect/soak | Evidence | Truth label |
| --- | --- | --- | --- | --- | --- | --- |
| _none tested_ | | | | | | Unproven |

## Evidence checklist

- exact hardware IDs and firmware where visible;
- OS, kernel, driver/BlueZ version, Grannus commit;
- configuration and exact reproduction steps;
- p50/p95/p99/sample count for latency claims;
- duration, disconnects, errors, and dropped/stale work for soak claims;
- logs/traces sanitized of secrets and personal identifiers.
