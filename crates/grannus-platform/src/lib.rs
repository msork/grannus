//! Platform interfaces and deterministic fake backends.

#![forbid(unsafe_code)]

pub use grannus_core::MonoTime;
use grannus_core::{ControllerState, TraceId};
use std::fmt;
use std::time::Duration;

/// Capture pixel layout. Backends may extend this capability list later.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    /// Packed YUYV 4:2:2.
    Yuyv,
    /// Motion JPEG from common UVC devices.
    Mjpeg,
    /// NV12 4:2:0 semi-planar.
    Nv12,
}

/// Negotiated capture format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VideoFormat {
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
    /// Frames-per-second numerator.
    pub fps_numerator: u16,
    /// Frames-per-second denominator.
    pub fps_denominator: u16,
    /// Pixel layout.
    pub pixel_format: PixelFormat,
}

impl VideoFormat {
    /// Validates dimensions and frame rate.
    ///
    /// # Errors
    /// Returns [`BackendError::InvalidConfiguration`] for zero-valued fields.
    pub const fn validate(self) -> Result<Self, BackendError> {
        if self.width == 0
            || self.height == 0
            || self.fps_numerator == 0
            || self.fps_denominator == 0
        {
            Err(BackendError::InvalidConfiguration)
        } else {
            Ok(self)
        }
    }
}

/// Captured frame metadata and payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedFrame {
    /// Non-secret trace identifier for correlating pipeline stage events.
    pub trace_id: TraceId,
    /// Monotonic capture-arrival time.
    pub captured_at: MonoTime,
    /// Negotiated format.
    pub format: VideoFormat,
    /// Backend sequence number.
    pub sequence: u64,
    /// Owned payload for the fake/reference path. Real zero-copy backends will
    /// expose a capability-tagged buffer handle behind a separate type.
    pub bytes: Vec<u8>,
}

/// Generic backend failure without leaking a vendor API into core layers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendError {
    /// Supplied configuration is invalid.
    InvalidConfiguration,
    /// Backend is not currently available.
    Unavailable,
    /// End of a deterministic fake source.
    EndOfStream,
    /// Device I/O failed.
    DeviceIo,
}

impl fmt::Display for BackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidConfiguration => "invalid backend configuration",
            Self::Unavailable => "backend unavailable",
            Self::EndOfStream => "end of stream",
            Self::DeviceIo => "device I/O failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for BackendError {}

/// Pull-style capture boundary. Blocking device implementations must run on a
/// dedicated thread rather than an async executor worker.
pub trait CaptureBackend {
    /// Returns the active format.
    fn format(&self) -> VideoFormat;

    /// Returns the next captured frame.
    ///
    /// # Errors
    /// Returns a backend-specific category without exposing vendor error types.
    fn next_frame(&mut self) -> Result<CapturedFrame, BackendError>;
}

/// Console-facing controller output boundary.
pub trait ControllerBackend {
    /// Emits the newest normalized state at the backend report cadence.
    ///
    /// # Errors
    /// Returns an error when the controller/device cannot accept state.
    fn send_state(&mut self, state: ControllerState) -> Result<(), BackendError>;

    /// Sends a neutral state before disconnect or after input timeout.
    ///
    /// # Errors
    /// Returns an error when the controller/device cannot accept state.
    fn neutralize(&mut self) -> Result<(), BackendError> {
        self.send_state(ControllerState::default())
    }
}

/// Deterministic capture source for tests and architecture bring-up.
#[derive(Debug)]
pub struct FakeCapture {
    format: VideoFormat,
    remaining: u64,
    sequence: u64,
}

impl FakeCapture {
    /// Creates a fake source producing exactly `frames` empty frames.
    ///
    /// # Errors
    /// Returns an error if the format is invalid.
    pub fn new(format: VideoFormat, frames: u64) -> Result<Self, BackendError> {
        Ok(Self {
            format: format.validate()?,
            remaining: frames,
            sequence: 0,
        })
    }
}

impl CaptureBackend for FakeCapture {
    fn format(&self) -> VideoFormat {
        self.format
    }

    fn next_frame(&mut self) -> Result<CapturedFrame, BackendError> {
        if self.remaining == 0 {
            return Err(BackendError::EndOfStream);
        }
        let sequence = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        self.remaining -= 1;
        Ok(CapturedFrame {
            trace_id: TraceId::new(sequence),
            captured_at: MonoTime::from_duration(Duration::from_micros(
                sequence.saturating_mul(16_667),
            )),
            format: self.format,
            sequence,
            bytes: Vec::new(),
        })
    }
}

/// Fake controller retaining only its most recently accepted state.
#[derive(Debug, Default)]
pub struct FakeController {
    latest: Option<ControllerState>,
    reports: u64,
}

impl FakeController {
    /// Returns the newest accepted state.
    #[must_use]
    pub const fn latest(&self) -> Option<ControllerState> {
        self.latest
    }

    /// Returns the number of emitted reports.
    #[must_use]
    pub const fn reports(&self) -> u64 {
        self.reports
    }
}

impl ControllerBackend for FakeController {
    fn send_state(&mut self, state: ControllerState) -> Result<(), BackendError> {
        self.latest = Some(state);
        self.reports = self.reports.saturating_add(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format() -> VideoFormat {
        VideoFormat {
            width: 1_920,
            height: 1_080,
            fps_numerator: 60,
            fps_denominator: 1,
            pixel_format: PixelFormat::Yuyv,
        }
    }

    #[test]
    fn fake_capture_is_finite_and_deterministic() {
        let mut capture = FakeCapture::new(format(), 2).unwrap();
        assert_eq!(capture.next_frame().unwrap().sequence, 0);
        assert_eq!(capture.next_frame().unwrap().sequence, 1);
        assert_eq!(capture.next_frame(), Err(BackendError::EndOfStream));
    }

    #[test]
    fn fake_controller_keeps_latest_state() {
        let mut controller = FakeController::default();
        let state = ControllerState::default();
        controller.send_state(state).unwrap();
        assert_eq!(controller.latest(), Some(state));
        assert_eq!(controller.reports(), 1);
    }

    #[test]
    fn invalid_video_format_is_rejected() {
        let mut invalid = format();
        invalid.fps_denominator = 0;
        assert_eq!(invalid.validate(), Err(BackendError::InvalidConfiguration));
    }
}
