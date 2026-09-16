//! Platform-neutral domain types and invariants for Grannus.

#![forbid(unsafe_code)]

use core::fmt;

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
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Signed normalized analog axis spanning the full i16 range.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Axis(i16);

impl Axis {
    /// Neutral axis value.
    pub const NEUTRAL: Self = Self(0);

    /// Creates an axis from an already normalized value.
    pub const fn new(value: i16) -> Self {
        Self(value)
    }

    /// Returns the normalized value.
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
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// Returns the normalized value.
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

/// Logical controller buttons independent of SDL, XInput, evdev, and brands.
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
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether a logical button is pressed.
    pub const fn contains(self, button: Button) -> bool {
        self.0 & (1_u32 << button as u8) != 0
    }

    /// Returns a new set with a logical button pressed or released.
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
            Self::PlayerSlotOutOfRange(value) => write!(formatter, "player slot {value} is invalid"),
            Self::UnknownButtonBits(bits) => write!(formatter, "unknown button bits: {bits:#x}"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

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
}
