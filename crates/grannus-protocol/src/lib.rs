//! Bounded, versioned wire representations shared by Grannus and Sirona.

#![forbid(unsafe_code)]

use core::fmt;
use grannus_core::{Axis, Buttons, ControllerState, PlayerSlot, Trigger, Vector3};

/// Current controller datagram version.
pub const INPUT_VERSION: u8 = 1;
/// Exact encoded size of a v1 input datagram.
pub const INPUT_DATAGRAM_V1_LEN: usize = 48;
const MAGIC: [u8; 2] = *b"GI";

/// A complete controller state sample transported with latest-state-wins semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputDatagram {
    /// Wrapping source sequence number.
    pub sequence: u32,
    /// Source monotonic timestamp in microseconds; meaningful only within a session.
    pub source_time_us: u64,
    /// Authorized target player slot.
    pub player_slot: PlayerSlot,
    /// Complete normalized controller state.
    pub state: ControllerState,
}

/// Input datagram decoding failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// Datagram length differs from the fixed version length.
    InvalidLength(usize),
    /// Datagram magic does not identify controller input.
    InvalidMagic,
    /// Protocol version is not supported.
    UnsupportedVersion(u8),
    /// Header flags contain reserved bits.
    InvalidFlags(u8),
    /// Reserved extension bytes are nonzero.
    ReservedNonZero,
    /// Player slot is invalid.
    InvalidPlayerSlot(u8),
    /// Button field contains reserved bits.
    InvalidButtonBits(u32),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(value) => write!(formatter, "invalid datagram length: {value}"),
            Self::InvalidMagic => formatter.write_str("invalid input datagram magic"),
            Self::UnsupportedVersion(value) => {
                write!(formatter, "unsupported input version: {value}")
            }
            Self::InvalidFlags(value) => write!(formatter, "invalid input flags: {value:#x}"),
            Self::ReservedNonZero => formatter.write_str("reserved input bytes are nonzero"),
            Self::InvalidPlayerSlot(value) => write!(formatter, "invalid player slot: {value}"),
            Self::InvalidButtonBits(value) => write!(formatter, "invalid button bits: {value:#x}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Encodes the fixed-size v1 datagram in network byte order.
#[must_use]
pub fn encode_input_v1(input: &InputDatagram) -> [u8; INPUT_DATAGRAM_V1_LEN] {
    let mut bytes = [0_u8; INPUT_DATAGRAM_V1_LEN];
    bytes[0..2].copy_from_slice(&MAGIC);
    bytes[2] = INPUT_VERSION;
    bytes[3] = u8::from(input.state.motion_valid);
    bytes[4..8].copy_from_slice(&input.sequence.to_be_bytes());
    bytes[8..16].copy_from_slice(&input.source_time_us.to_be_bytes());
    bytes[16] = input.player_slot.get();
    bytes[17..21].copy_from_slice(&input.state.buttons.bits().to_be_bytes());
    write_i16(&mut bytes, 21, input.state.left_stick[0].get());
    write_i16(&mut bytes, 23, input.state.left_stick[1].get());
    write_i16(&mut bytes, 25, input.state.right_stick[0].get());
    write_i16(&mut bytes, 27, input.state.right_stick[1].get());
    write_u16(&mut bytes, 29, input.state.left_trigger.get());
    write_u16(&mut bytes, 31, input.state.right_trigger.get());
    write_vector(&mut bytes, 33, input.state.gyro);
    write_vector(&mut bytes, 39, input.state.accelerometer);
    // bytes 45..48 are reserved zero for compatible v1 extensions.
    bytes
}

/// Decodes and validates one fixed-size v1 controller datagram.
///
/// # Errors
/// Returns [`DecodeError`] for invalid length, identity, version, flags, or fields.
pub fn decode_input_v1(bytes: &[u8]) -> Result<InputDatagram, DecodeError> {
    if bytes.len() != INPUT_DATAGRAM_V1_LEN {
        return Err(DecodeError::InvalidLength(bytes.len()));
    }
    if bytes[0..2] != MAGIC {
        return Err(DecodeError::InvalidMagic);
    }
    if bytes[2] != INPUT_VERSION {
        return Err(DecodeError::UnsupportedVersion(bytes[2]));
    }
    if bytes[3] & !1 != 0 {
        return Err(DecodeError::InvalidFlags(bytes[3]));
    }
    if bytes[45..48] != [0, 0, 0] {
        return Err(DecodeError::ReservedNonZero);
    }

    let slot = PlayerSlot::new(bytes[16]).map_err(|_| DecodeError::InvalidPlayerSlot(bytes[16]))?;
    let button_bits = read_u32(bytes, 17);
    let buttons =
        Buttons::from_bits(button_bits).map_err(|_| DecodeError::InvalidButtonBits(button_bits))?;

    Ok(InputDatagram {
        sequence: read_u32(bytes, 4),
        source_time_us: read_u64(bytes, 8),
        player_slot: slot,
        state: ControllerState {
            buttons,
            left_stick: [
                Axis::new(read_i16(bytes, 21)),
                Axis::new(read_i16(bytes, 23)),
            ],
            right_stick: [
                Axis::new(read_i16(bytes, 25)),
                Axis::new(read_i16(bytes, 27)),
            ],
            left_trigger: Trigger::new(read_u16(bytes, 29)),
            right_trigger: Trigger::new(read_u16(bytes, 31)),
            gyro: read_vector(bytes, 33),
            accelerometer: read_vector(bytes, 39),
            motion_valid: bytes[3] & 1 == 1,
        },
    })
}

fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn write_vector(bytes: &mut [u8], offset: usize, vector: Vector3) {
    write_i16(bytes, offset, vector.x);
    write_i16(bytes, offset + 2, vector.y);
    write_i16(bytes, offset + 4, vector.z);
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("validated fixed length"),
    )
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("validated fixed length"),
    )
}

fn read_vector(bytes: &[u8], offset: usize) -> Vector3 {
    Vector3 {
        x: read_i16(bytes, offset),
        y: read_i16(bytes, offset + 2),
        z: read_i16(bytes, offset + 4),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grannus_core::Button;

    fn sample() -> InputDatagram {
        InputDatagram {
            sequence: 42,
            source_time_us: 1_234_567,
            player_slot: PlayerSlot::new(2).unwrap(),
            state: ControllerState {
                buttons: Buttons::default()
                    .with(Button::South, true)
                    .with(Button::Home, true),
                left_stick: [Axis::new(i16::MIN), Axis::new(i16::MAX)],
                right_stick: [Axis::new(-123), Axis::new(456)],
                left_trigger: Trigger::new(12_345),
                right_trigger: Trigger::new(u16::MAX),
                gyro: Vector3 { x: -1, y: 2, z: -3 },
                accelerometer: Vector3 { x: 4, y: -5, z: 6 },
                motion_valid: true,
            },
        }
    }

    #[test]
    fn v1_round_trip_preserves_state() {
        let expected = sample();
        assert_eq!(decode_input_v1(&encode_input_v1(&expected)), Ok(expected));
    }

    #[test]
    fn rejects_wrong_sizes_without_panicking() {
        for length in 0..INPUT_DATAGRAM_V1_LEN {
            assert_eq!(
                decode_input_v1(&vec![0; length]),
                Err(DecodeError::InvalidLength(length))
            );
        }
        assert_eq!(
            decode_input_v1(&[0; INPUT_DATAGRAM_V1_LEN + 1]),
            Err(DecodeError::InvalidLength(INPUT_DATAGRAM_V1_LEN + 1))
        );
    }

    #[test]
    fn rejects_reserved_fields() {
        let mut bytes = encode_input_v1(&sample());
        bytes[45] = 1;
        assert_eq!(decode_input_v1(&bytes), Err(DecodeError::ReservedNonZero));
    }

    #[test]
    fn rejects_unknown_buttons() {
        let mut bytes = encode_input_v1(&sample());
        bytes[17..21].copy_from_slice(&(1_u32 << 31).to_be_bytes());
        assert_eq!(
            decode_input_v1(&bytes),
            Err(DecodeError::InvalidButtonBits(1 << 31))
        );
    }
}
