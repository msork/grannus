# Threat model

Status: initial model for the foundation. Revisit before any Internet traversal.

## Assets

- ability to control the attached console;
- capture-card video/audio and user privacy;
- paired client identities and session secrets;
- host integrity, availability, and network position;
- player-slot ownership and haptic output;
- diagnostic data that may reveal devices, addresses, or activity.

## Trust boundaries

The network, Sirona input, discovery packets, capture-card metadata, Bluetooth
traffic, configuration files, and diagnostic imports are untrusted. A paired
client is authenticated but not trusted to send well-formed or rate-limited
data. Native/vendor APIs are memory-safety and availability boundaries.

## Primary threats and controls

| Threat | Required controls |
| --- | --- |
| unauthorized viewing/control | explicit pairing, persistent identity, mutual session auth, least privilege |
| slot hijacking | authorized leases, identity-bound reconnect, admin approval/conflict rules |
| packet forgery/replay | established AEAD transport, session nonce/key separation, replay window |
| parser/resource exhaustion | fixed maxima, bounded queues, early validation, fuzzing, rate limits |
| stale input after loss/reorder | sequence window, deadlines, neutral-state timeout |
| discovery/pairing abuse | local/limited scope, user-visible code, rate limit, expiry, no secret logs |
| UPnP/NAT exposure | opt-in authenticated endpoint only, visible/revocable leases, no admin plane |
| secret theft | OS secure storage or mode-0600 files, zero secret logging, rotation/revocation |
| malicious device/backend | constrained privileges, narrow interfaces, restart bounds, input validation |
| diagnostics leakage | explicit export, sanitization, payload capture off by default |

## Security invariants

1. No controller state reaches a backend without an authenticated identity and
   active authorized slot lease once network sessions exist.
2. Discovery never grants access.
3. Loss of authorization/disconnect eventually emits neutral state.
4. Unknown versions and reserved mandatory bits fail closed.
5. Memory and queued work remain bounded under hostile traffic.
6. Media reliability cannot block newer controller input.
7. NAT traversal cannot silently weaken authentication.

## Out of scope for pre-alpha

Internet exposure, relay operation, multi-tenant hosting, and hostile local root
are unsupported. This does not excuse safe parsing or secret handling.

## Review triggers

Update this model when selecting transport/crypto, adding pairing/storage,
opening a listener, enabling discovery/NAT traversal, loading plugins/vendor
SDKs, exporting metrics remotely, or changing slot ownership.
