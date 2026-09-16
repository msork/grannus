//! Platform-neutral domain types and invariants for Grannus.

#![forbid(unsafe_code)]

use core::fmt;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

/// Monotonic timestamp relative to a process-local epoch.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MonoTime(Duration);

impl MonoTime {
    /// Constructs a process-relative timestamp for deterministic sources and tests.
    #[must_use]
    pub const fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    /// Returns the process-relative duration.
    #[must_use]
    pub const fn duration(self) -> Duration {
        self.0
    }
}

/// Process-local monotonic clock for timestamping host stage boundaries.
#[derive(Debug)]
pub struct MonoClock {
    epoch: Instant,
}

impl MonoClock {
    /// Starts a clock at the current monotonic instant.
    #[must_use]
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }

    /// Returns the duration since this clock's process-local epoch.
    #[must_use]
    pub fn now(&self) -> MonoTime {
        MonoTime::from_duration(self.epoch.elapsed())
    }
}

impl Default for MonoClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Non-secret identifier that correlates stage events for one work item.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TraceId(u64);

impl TraceId {
    /// Creates a trace identifier supplied by the owning lifecycle component.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the opaque identifier for diagnostics export.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A named pipeline boundary at which a monotonic timestamp is recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetricStage {
    /// A capture backend made a frame available to the host.
    CaptureArrival,
    /// The host accepted a normalized input snapshot.
    InputAccepted,
    /// A controller backend emitted a state report.
    ControllerReport,
}

/// One histogram-ready timestamped observation at a pipeline boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetricEvent {
    /// Correlates this observation with events for the same frame or input sample.
    pub trace_id: TraceId,
    /// Pipeline boundary that emitted the observation.
    pub stage: MetricStage,
    /// Process-local monotonic time at the boundary.
    pub at: MonoTime,
}

impl MetricEvent {
    /// Creates a timestamped stage observation.
    #[must_use]
    pub const fn new(trace_id: TraceId, stage: MetricStage, at: MonoTime) -> Self {
        Self {
            trace_id,
            stage,
            at,
        }
    }
}

/// Bounded newest-first-retaining storage for metric events.
///
/// On overflow, the oldest event is discarded and counted so a slow diagnostics
/// consumer cannot block or retain stale pipeline observations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricEventBuffer {
    capacity: NonZeroUsize,
    events: VecDeque<MetricEvent>,
    dropped_events: u64,
}

impl MetricEventBuffer {
    /// Creates an empty buffer with a fixed nonzero capacity.
    #[must_use]
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity,
            events: VecDeque::with_capacity(capacity.get()),
            dropped_events: 0,
        }
    }

    /// Records one event, dropping the oldest event when the buffer is full.
    pub fn record(&mut self, event: MetricEvent) {
        if self.events.len() == self.capacity.get() {
            let _ = self.events.pop_front();
            self.dropped_events = self.dropped_events.saturating_add(1);
        }
        self.events.push_back(event);
    }

    /// Returns retained events from oldest to newest.
    #[must_use]
    pub fn events(&self) -> impl ExactSizeIterator<Item = &MetricEvent> {
        self.events.iter()
    }

    /// Returns how many events were discarded because the buffer was full.
    #[must_use]
    pub const fn dropped_events(&self) -> u64 {
        self.dropped_events
    }
}

/// Maximum player slots representable by the initial protocol.
pub const MAX_PLAYER_SLOTS: u8 = 8;

/// Stable player slot identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlayerSlot(u8);

impl PlayerSlot {
    /// Creates a validated zero-based player slot.
    ///
    /// # Errors
    /// Returns [`ValidationError::PlayerSlotOutOfRange`] for an unsupported slot.
    pub const fn new(value: u8) -> Result<Self, ValidationError> {
        if value < MAX_PLAYER_SLOTS {
            Ok(Self(value))
        } else {
            Err(ValidationError::PlayerSlotOutOfRange(value))
        }
    }

    /// Returns the zero-based value used on the wire.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Number of forward sequence values that are unambiguously newer under u32 wrapping.
pub const SEQUENCE_FORWARD_WINDOW: u32 = 1_u32 << 31;

/// Result of comparing a received snapshot sequence with the newest accepted sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceDecision {
    /// The snapshot advances the latest accepted sequence.
    Accepted,
    /// The snapshot repeats the newest accepted sequence.
    Duplicate,
    /// The snapshot is older, reordered, or exactly half a sequence space away.
    Stale,
}

/// Fixed-memory latest-state sequence window for one authenticated input source.
///
/// A sequence is newer when its wrapping forward distance is in
/// `1..SEQUENCE_FORWARD_WINDOW`. The exact half-range is intentionally rejected
/// because it has no unambiguous ordering. A session lifecycle event must call
/// [`SequenceWindow::reset`] before a new authenticated session starts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SequenceWindow {
    latest: Option<u32>,
}

impl SequenceWindow {
    /// Creates an empty sequence window that accepts its first sequence.
    #[must_use]
    pub const fn new() -> Self {
        Self { latest: None }
    }

    /// Classifies a received sequence and advances only for newer snapshots.
    pub fn observe(&mut self, sequence: u32) -> SequenceDecision {
        let decision = match self.latest {
            None => SequenceDecision::Accepted,
            Some(latest) if sequence == latest => SequenceDecision::Duplicate,
            Some(latest) if is_newer_sequence(sequence, latest) => SequenceDecision::Accepted,
            Some(_) => SequenceDecision::Stale,
        };
        if decision == SequenceDecision::Accepted {
            self.latest = Some(sequence);
        }
        decision
    }

    /// Returns the newest accepted sequence, if any.
    #[must_use]
    pub const fn latest(self) -> Option<u32> {
        self.latest
    }

    /// Clears sequence state after an authenticated session lifecycle reset.
    pub fn reset(&mut self) {
        self.latest = None;
    }
}

/// Returns whether `candidate` is unambiguously newer than `reference`.
#[must_use]
pub const fn is_newer_sequence(candidate: u32, reference: u32) -> bool {
    let distance = candidate.wrapping_sub(reference);
    distance != 0 && distance < SEQUENCE_FORWARD_WINDOW
}

/// Opaque, authenticated client identity supplied by the session layer.
///
/// This identifier is not a secret and does not perform authentication. The
/// session layer must validate its provenance before calling [`SlotManager`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ClientIdentity(u64);

impl ClientIdentity {
    /// Creates an opaque identity for lease ownership checks.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the opaque identity value for sanitized diagnostics.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Lifecycle state of a player-slot lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeaseState {
    /// The owner has an active session and may submit input for this slot.
    Active,
    /// The owner disconnected and may reclaim the slot during a configured grace period.
    ReconnectGrace,
}

/// Explicit ownership of one player slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlotLease {
    slot: PlayerSlot,
    owner: ClientIdentity,
    state: LeaseState,
}

impl SlotLease {
    /// Returns the leased player slot.
    #[must_use]
    pub const fn slot(self) -> PlayerSlot {
        self.slot
    }

    /// Returns the identity currently entitled to this lease.
    #[must_use]
    pub const fn owner(self) -> ClientIdentity {
        self.owner
    }

    /// Returns whether this lease currently accepts input.
    #[must_use]
    pub const fn state(self) -> LeaseState {
        self.state
    }
}

/// Fixed-capacity player-slot ownership state machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotManager {
    leases: [Option<SlotLease>; MAX_PLAYER_SLOTS as usize],
}

impl SlotManager {
    /// Creates a manager with no assigned slots.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            leases: [None; MAX_PLAYER_SLOTS as usize],
        }
    }

    /// Claims a free slot or reactivates the same owner's reconnect-grace lease.
    ///
    /// # Errors
    /// Returns [`SlotError`] when the slot belongs to another identity or the
    /// identity already owns a different slot.
    pub fn claim(
        &mut self,
        slot: PlayerSlot,
        identity: ClientIdentity,
    ) -> Result<SlotLease, SlotError> {
        let index = usize::from(slot.get());
        if let Some(lease) = self.leases[index] {
            if lease.owner != identity {
                return Err(SlotError::SlotOccupied {
                    slot,
                    owner: lease.owner,
                });
            }
            let active = SlotLease {
                state: LeaseState::Active,
                ..lease
            };
            self.leases[index] = Some(active);
            return Ok(active);
        }

        if let Some(existing) = self
            .leases
            .iter()
            .flatten()
            .find(|lease| lease.owner == identity)
        {
            return Err(SlotError::IdentityAlreadyLeased {
                identity,
                slot: existing.slot,
            });
        }

        let lease = SlotLease {
            slot,
            owner: identity,
            state: LeaseState::Active,
        };
        self.leases[index] = Some(lease);
        Ok(lease)
    }

    /// Marks an active lease as reclaimable only by its current owner.
    ///
    /// # Errors
    /// Returns [`SlotError::NoLeaseForIdentity`] when the identity has no lease,
    /// or [`SlotError::AlreadyDisconnected`] when it is already in grace.
    pub fn disconnect(&mut self, identity: ClientIdentity) -> Result<SlotLease, SlotError> {
        let Some((index, lease)) = self.leases.iter().enumerate().find_map(|(index, lease)| {
            lease
                .filter(|lease| lease.owner == identity)
                .map(|lease| (index, lease))
        }) else {
            return Err(SlotError::NoLeaseForIdentity(identity));
        };
        if lease.state == LeaseState::ReconnectGrace {
            return Err(SlotError::AlreadyDisconnected { slot: lease.slot });
        }
        let disconnected = SlotLease {
            state: LeaseState::ReconnectGrace,
            ..lease
        };
        self.leases[index] = Some(disconnected);
        Ok(disconnected)
    }

    /// Releases a reconnect-grace lease after its externally enforced grace deadline.
    ///
    /// # Errors
    /// Returns [`SlotError`] when there is no lease or when the owner is still active.
    pub fn expire_reconnect(&mut self, slot: PlayerSlot) -> Result<SlotLease, SlotError> {
        let index = usize::from(slot.get());
        let lease = self.leases[index].ok_or(SlotError::NoLease { slot })?;
        if lease.state == LeaseState::Active {
            return Err(SlotError::ActiveLease { slot });
        }
        self.leases[index] = None;
        Ok(lease)
    }

    /// Releases a lease only when requested by its current owner.
    ///
    /// # Errors
    /// Returns [`SlotError`] when the slot is unassigned or owned by another identity.
    pub fn release(
        &mut self,
        slot: PlayerSlot,
        identity: ClientIdentity,
    ) -> Result<SlotLease, SlotError> {
        let index = usize::from(slot.get());
        let lease = self.leases[index].ok_or(SlotError::NoLease { slot })?;
        if lease.owner != identity {
            return Err(SlotError::NotOwner {
                slot,
                owner: lease.owner,
            });
        }
        self.leases[index] = None;
        Ok(lease)
    }

    /// Authorizes an input source only when it owns the active target slot.
    ///
    /// # Errors
    /// Returns [`SlotError`] when the slot is absent, belongs to another identity,
    /// or is awaiting reconnect.
    pub fn authorize(
        &self,
        slot: PlayerSlot,
        identity: ClientIdentity,
    ) -> Result<SlotLease, SlotError> {
        let lease = self.leases[usize::from(slot.get())].ok_or(SlotError::NoLease { slot })?;
        if lease.owner != identity {
            return Err(SlotError::NotOwner {
                slot,
                owner: lease.owner,
            });
        }
        if lease.state == LeaseState::ReconnectGrace {
            return Err(SlotError::ReconnectGrace { slot });
        }
        Ok(lease)
    }

    /// Returns the current lease for a slot, including reconnect-grace leases.
    #[must_use]
    pub const fn lease(&self, slot: PlayerSlot) -> Option<SlotLease> {
        self.leases[slot.get() as usize]
    }
}

impl Default for SlotManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Slot-ownership transition or authorization failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotError {
    /// The requested slot belongs to a different identity.
    SlotOccupied {
        /// Requested player slot.
        slot: PlayerSlot,
        /// Current lease owner.
        owner: ClientIdentity,
    },
    /// The identity already owns a different slot.
    IdentityAlreadyLeased {
        /// Identity attempting a second claim.
        identity: ClientIdentity,
        /// Existing owned slot.
        slot: PlayerSlot,
    },
    /// The identity does not own any slot.
    NoLeaseForIdentity(ClientIdentity),
    /// The requested slot does not have a lease.
    NoLease {
        /// Requested player slot.
        slot: PlayerSlot,
    },
    /// A different identity attempted an owner-only operation or input.
    NotOwner {
        /// Requested player slot.
        slot: PlayerSlot,
        /// Current lease owner.
        owner: ClientIdentity,
    },
    /// The lease is already in reconnect grace.
    AlreadyDisconnected {
        /// Leased player slot.
        slot: PlayerSlot,
    },
    /// An active lease cannot be expired.
    ActiveLease {
        /// Leased player slot.
        slot: PlayerSlot,
    },
    /// Input was submitted while the owner is in reconnect grace.
    ReconnectGrace {
        /// Leased player slot.
        slot: PlayerSlot,
    },
}

impl fmt::Display for SlotError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SlotOccupied { slot, .. } => {
                write!(formatter, "slot {} is already leased", slot.get())
            }
            Self::IdentityAlreadyLeased { slot, .. } => {
                write!(formatter, "identity already owns slot {}", slot.get())
            }
            Self::NoLeaseForIdentity(_) => formatter.write_str("identity does not own a slot"),
            Self::NoLease { slot } => write!(formatter, "slot {} has no lease", slot.get()),
            Self::NotOwner { slot, .. } => {
                write!(formatter, "identity does not own slot {}", slot.get())
            }
            Self::AlreadyDisconnected { slot } => {
                write!(formatter, "slot {} is already disconnected", slot.get())
            }
            Self::ActiveLease { slot } => write!(formatter, "slot {} is still active", slot.get()),
            Self::ReconnectGrace { slot } => {
                write!(formatter, "slot {} is awaiting reconnect", slot.get())
            }
        }
    }
}

impl std::error::Error for SlotError {}

/// Signed normalized analog axis spanning the full i16 range.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Axis(i16);

impl Axis {
    /// Neutral axis value.
    pub const NEUTRAL: Self = Self(0);

    /// Creates an axis from an already normalized value.
    #[must_use]
    pub const fn new(value: i16) -> Self {
        Self(value)
    }

    /// Returns the normalized value.
    #[must_use]
    pub const fn get(self) -> i16 {
        self.0
    }
}

/// Unsigned normalized trigger spanning zero to `u16::MAX`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Trigger(u16);

impl Trigger {
    /// Released trigger value.
    pub const RELEASED: Self = Self(0);

    /// Creates a trigger from an already normalized value.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the normalized value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Three-axis inertial sample in protocol-normalized fixed-point units.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Vector3 {
    /// X component.
    pub x: i16,
    /// Y component.
    pub y: i16,
    /// Z component.
    pub z: i16,
}

/// Logical controller buttons independent of SDL, `XInput`, evdev, and brands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Button {
    /// South face button.
    South = 0,
    /// East face button.
    East = 1,
    /// West face button.
    West = 2,
    /// North face button.
    North = 3,
    /// D-pad up.
    DpadUp = 4,
    /// D-pad down.
    DpadDown = 5,
    /// D-pad left.
    DpadLeft = 6,
    /// D-pad right.
    DpadRight = 7,
    /// Left shoulder.
    LeftShoulder = 8,
    /// Right shoulder.
    RightShoulder = 9,
    /// Left stick click.
    LeftStick = 10,
    /// Right stick click.
    RightStick = 11,
    /// Left auxiliary/menu button.
    Select = 12,
    /// Right auxiliary/menu button.
    Start = 13,
    /// System/home button.
    Home = 14,
    /// Capture/share button.
    Capture = 15,
    /// Left digital trigger button.
    LeftTrigger = 16,
    /// Right digital trigger button.
    RightTrigger = 17,
}

/// Compact logical button set. Unassigned bits must remain zero on the wire.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Buttons(u32);

impl Buttons {
    /// Bits currently assigned to known buttons.
    pub const KNOWN_MASK: u32 = (1_u32 << 18) - 1;

    /// Creates a button set after checking reserved bits.
    ///
    /// # Errors
    /// Returns [`ValidationError::UnknownButtonBits`] when reserved bits are set.
    pub const fn from_bits(bits: u32) -> Result<Self, ValidationError> {
        let unknown = bits & !Self::KNOWN_MASK;
        if unknown == 0 {
            Ok(Self(bits))
        } else {
            Err(ValidationError::UnknownButtonBits(unknown))
        }
    }

    /// Returns the encoded bits.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether a logical button is pressed.
    #[must_use]
    pub const fn contains(self, button: Button) -> bool {
        self.0 & (1_u32 << button as u8) != 0
    }

    /// Returns a new set with a logical button pressed or released.
    #[must_use]
    pub const fn with(self, button: Button, pressed: bool) -> Self {
        let bit = 1_u32 << button as u8;
        if pressed {
            Self(self.0 | bit)
        } else {
            Self(self.0 & !bit)
        }
    }
}

/// Complete normalized controller snapshot. Fresh snapshots supersede old ones.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ControllerState {
    /// Digital buttons.
    pub buttons: Buttons,
    /// Left stick X/Y.
    pub left_stick: [Axis; 2],
    /// Right stick X/Y.
    pub right_stick: [Axis; 2],
    /// Left analog trigger.
    pub left_trigger: Trigger,
    /// Right analog trigger.
    pub right_trigger: Trigger,
    /// Gyroscope sample when `motion_valid` is true.
    pub gyro: Vector3,
    /// Accelerometer sample when `motion_valid` is true.
    pub accelerometer: Vector3,
    /// Whether inertial fields are meaningful.
    pub motion_valid: bool,
}

/// Domain validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// Player slot is outside the supported range.
    PlayerSlotOutOfRange(u8),
    /// Reserved button bits were set.
    UnknownButtonBits(u32),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlayerSlotOutOfRange(value) => {
                write!(formatter, "player slot {value} is invalid")
            }
            Self::UnknownButtonBits(bits) => write!(formatter, "unknown button bits: {bits:#x}"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn player_slots_are_bounded() {
        assert_eq!(PlayerSlot::new(0).map(PlayerSlot::get), Ok(0));
        assert_eq!(PlayerSlot::new(7).map(PlayerSlot::get), Ok(7));
        assert_eq!(
            PlayerSlot::new(8),
            Err(ValidationError::PlayerSlotOutOfRange(8))
        );
    }

    #[test]
    fn buttons_reject_reserved_bits() {
        assert!(Buttons::from_bits(Buttons::KNOWN_MASK).is_ok());
        assert_eq!(
            Buttons::from_bits(1 << 31),
            Err(ValidationError::UnknownButtonBits(1 << 31))
        );
    }

    #[test]
    fn button_mutation_is_value_semantic() {
        let empty = Buttons::default();
        let pressed = empty.with(Button::South, true);
        assert!(!empty.contains(Button::South));
        assert!(pressed.contains(Button::South));
        assert!(!pressed.with(Button::South, false).contains(Button::South));
    }

    #[test]
    fn monotonic_clock_never_moves_backwards() {
        let clock = MonoClock::new();
        let first = clock.now();
        let second = clock.now();
        assert!(second >= first);
    }

    #[test]
    fn metric_buffer_keeps_newest_events_and_counts_drops() {
        let mut events = MetricEventBuffer::new(NonZeroUsize::new(2).unwrap());
        let trace = TraceId::new(7);
        events.record(MetricEvent::new(
            trace,
            MetricStage::CaptureArrival,
            MonoTime::from_duration(Duration::from_micros(1)),
        ));
        events.record(MetricEvent::new(
            trace,
            MetricStage::InputAccepted,
            MonoTime::from_duration(Duration::from_micros(2)),
        ));
        events.record(MetricEvent::new(
            trace,
            MetricStage::ControllerReport,
            MonoTime::from_duration(Duration::from_micros(3)),
        ));

        assert_eq!(events.dropped_events(), 1);
        assert_eq!(
            events.events().copied().collect::<Vec<_>>(),
            [
                MetricEvent::new(
                    trace,
                    MetricStage::InputAccepted,
                    MonoTime::from_duration(Duration::from_micros(2)),
                ),
                MetricEvent::new(
                    trace,
                    MetricStage::ControllerReport,
                    MonoTime::from_duration(Duration::from_micros(3)),
                ),
            ]
        );
    }

    #[test]
    fn reconnect_reclaims_only_the_original_identity() {
        let slot = PlayerSlot::new(3).unwrap();
        let owner = ClientIdentity::new(10);
        let contender = ClientIdentity::new(20);
        let mut slots = SlotManager::new();

        assert_eq!(
            slots.claim(slot, owner).unwrap().state(),
            LeaseState::Active
        );
        assert_eq!(
            slots.claim(slot, contender),
            Err(SlotError::SlotOccupied { slot, owner })
        );
        assert_eq!(
            slots.disconnect(owner).unwrap().state(),
            LeaseState::ReconnectGrace
        );
        assert_eq!(
            slots.authorize(slot, owner),
            Err(SlotError::ReconnectGrace { slot })
        );
        assert_eq!(
            slots.claim(slot, contender),
            Err(SlotError::SlotOccupied { slot, owner })
        );
        assert_eq!(
            slots.claim(slot, owner).unwrap().state(),
            LeaseState::Active
        );
        assert!(slots.authorize(slot, owner).is_ok());
    }

    #[test]
    fn only_disconnected_leases_expire_before_reassignment() {
        let slot = PlayerSlot::new(0).unwrap();
        let owner = ClientIdentity::new(1);
        let replacement = ClientIdentity::new(2);
        let mut slots = SlotManager::new();

        slots.claim(slot, owner).unwrap();
        assert_eq!(
            slots.expire_reconnect(slot),
            Err(SlotError::ActiveLease { slot })
        );
        slots.disconnect(owner).unwrap();
        assert_eq!(slots.expire_reconnect(slot).unwrap().owner(), owner);
        assert_eq!(slots.claim(slot, replacement).unwrap().owner(), replacement);
    }

    #[test]
    fn owner_checks_and_fixed_slot_bound_reject_invalid_operations() {
        let owner = ClientIdentity::new(1);
        let other = ClientIdentity::new(2);
        let first = PlayerSlot::new(0).unwrap();
        let second = PlayerSlot::new(1).unwrap();
        let mut slots = SlotManager::new();

        slots.claim(first, owner).unwrap();
        assert_eq!(
            slots.release(first, other),
            Err(SlotError::NotOwner { slot: first, owner })
        );
        assert_eq!(
            slots.claim(second, owner),
            Err(SlotError::IdentityAlreadyLeased {
                identity: owner,
                slot: first,
            })
        );
        assert_eq!(
            slots.authorize(first, other),
            Err(SlotError::NotOwner { slot: first, owner })
        );
        assert_eq!(
            PlayerSlot::new(MAX_PLAYER_SLOTS),
            Err(ValidationError::PlayerSlotOutOfRange(MAX_PLAYER_SLOTS))
        );
    }

    #[test]
    fn sequence_window_rejects_duplicate_reordered_and_ambiguous_sequences() {
        let mut window = SequenceWindow::new();
        assert_eq!(window.observe(u32::MAX), SequenceDecision::Accepted);
        assert_eq!(window.observe(0), SequenceDecision::Accepted);
        assert_eq!(window.observe(0), SequenceDecision::Duplicate);
        assert_eq!(window.observe(u32::MAX), SequenceDecision::Stale);
        assert_eq!(
            window.observe(SEQUENCE_FORWARD_WINDOW),
            SequenceDecision::Stale
        );
        assert_eq!(window.latest(), Some(0));
    }

    #[test]
    fn sequence_window_reset_requires_an_explicit_lifecycle_transition() {
        let mut window = SequenceWindow::new();
        assert_eq!(window.observe(100), SequenceDecision::Accepted);
        assert_eq!(window.observe(99), SequenceDecision::Stale);
        window.reset();
        assert_eq!(window.latest(), None);
        assert_eq!(window.observe(99), SequenceDecision::Accepted);
    }

    proptest! {
        #[test]
        fn all_unambiguous_forward_distances_advance_the_window(
            latest in any::<u32>(),
            distance in 1_u32..SEQUENCE_FORWARD_WINDOW,
        ) {
            let candidate = latest.wrapping_add(distance);
            let mut window = SequenceWindow::new();
            prop_assert_eq!(window.observe(latest), SequenceDecision::Accepted);
            prop_assert!(is_newer_sequence(candidate, latest));
            prop_assert_eq!(window.observe(candidate), SequenceDecision::Accepted);
            prop_assert_eq!(window.latest(), Some(candidate));
        }

        #[test]
        fn backward_and_ambiguous_distances_never_replace_latest(
            latest in any::<u32>(),
            distance in SEQUENCE_FORWARD_WINDOW..=u32::MAX,
        ) {
            let candidate = latest.wrapping_add(distance);
            let mut window = SequenceWindow::new();
            prop_assert_eq!(window.observe(latest), SequenceDecision::Accepted);
            prop_assert!(!is_newer_sequence(candidate, latest));
            prop_assert_eq!(window.observe(candidate), SequenceDecision::Stale);
            prop_assert_eq!(window.latest(), Some(latest));
        }
    }
}
