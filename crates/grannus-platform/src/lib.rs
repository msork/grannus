//! Platform interfaces and deterministic fake backends.

#![forbid(unsafe_code)]

pub use grannus_core::MonoTime;
use grannus_core::{ControllerState, TraceId};
use std::collections::VecDeque;
use std::fmt;
use std::num::NonZeroUsize;
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

/// Capabilities reported by a V4L2 device before buffer allocation.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V4l2Capabilities {
    pub streaming: bool,
    pub formats: Vec<PixelFormat>,
}

/// Configuration for a bounded V4L2 mmap capture session.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct V4l2MmapConfig {
    pub format: VideoFormat,
    pub buffer_count: NonZeroUsize,
}

impl V4l2MmapConfig {
    /// Validates negotiated format and bounds before any device allocation.
    ///
    /// # Errors
    /// Returns [`BackendError::InvalidConfiguration`] for invalid formats or
    /// more than eight mmap buffers.
    pub fn validate(self) -> Result<Self, BackendError> {
        self.format.validate()?;
        if self.buffer_count.get() > 8 {
            return Err(BackendError::InvalidConfiguration);
        }
        Ok(self)
    }
}

impl V4l2Capabilities {
    /// Selects a requested format only when the device advertises it.
    ///
    /// # Errors
    /// Returns [`BackendError::Unavailable`] when streaming or the pixel format
    /// is unsupported.
    pub fn negotiate(&self, requested: VideoFormat) -> Result<VideoFormat, BackendError> {
        let requested = requested.validate()?;
        if !self.streaming || !self.formats.contains(&requested.pixel_format) {
            return Err(BackendError::Unavailable);
        }
        Ok(requested)
    }
}

impl VideoFormat {
    /// Computes the maximum payload bytes for the negotiated pixel layout.
    ///
    /// # Errors
    /// Returns [`BackendError::InvalidConfiguration`] if multiplication would
    /// overflow the bounded representation.
    pub fn max_frame_bytes(self) -> Result<usize, BackendError> {
        self.validate()?;
        let pixels = usize::from(self.width)
            .checked_mul(usize::from(self.height))
            .ok_or(BackendError::InvalidConfiguration)?;
        let bytes = match self.pixel_format {
            PixelFormat::Yuyv | PixelFormat::Mjpeg => pixels.checked_mul(2),
            PixelFormat::Nv12 => pixels.checked_mul(3).and_then(|value| value.checked_div(2)),
        };
        bytes.ok_or(BackendError::InvalidConfiguration)
    }

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

/// Reports whether a capture timestamp regressed relative to the prior frame.
#[must_use]
pub fn timestamp_discontinuity(previous: Option<MonoTime>, current: MonoTime) -> bool {
    previous.is_some_and(|prior| current < prior)
}

/// Bounded tracker for capture timestamp continuity across frames.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimestampTracker {
    previous: Option<MonoTime>,
}

impl TimestampTracker {
    /// Creates an empty tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self { previous: None }
    }
    /// Records a timestamp and reports whether it regressed.
    pub fn observe(&mut self, current: MonoTime) -> bool {
        let discontinuity = timestamp_discontinuity(self.previous, current);
        self.previous = Some(current);
        discontinuity
    }
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

/// Encoded media produced without committing to a codec or wire protocol.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedFrame {
    pub sequence: u64,
    pub bytes: Vec<u8>,
}

/// Deterministic fake encoder for pipeline and queue tests.
#[derive(Debug, Default)]
pub struct FakeEncoder;

#[allow(missing_docs)]
impl FakeEncoder {
    #[must_use]
    pub fn encode(frame: CapturedFrame) -> EncodedFrame {
        EncodedFrame {
            sequence: frame.sequence,
            bytes: frame.bytes,
        }
    }
}

/// Fixed-capacity queue that drops the oldest media item on overflow.
#[derive(Debug)]
pub struct BoundedMediaQueue {
    capacity: usize,
    items: VecDeque<EncodedFrame>,
    dropped: u64,
}

#[allow(missing_docs)]
impl BoundedMediaQueue {
    #[must_use]
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            capacity: capacity.get(),
            items: VecDeque::with_capacity(capacity.get()),
            dropped: 0,
        }
    }
    pub fn push(&mut self, item: EncodedFrame) {
        if self.items.len() == self.capacity {
            self.items.pop_front();
            self.dropped = self.dropped.saturating_add(1);
        }
        self.items.push_back(item);
    }
    pub fn pop(&mut self) -> Option<EncodedFrame> {
        self.items.pop_front()
    }
    #[must_use]
    pub const fn dropped(&self) -> u64 {
        self.dropped
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Fake transport with deterministic loss, reordering, and reconnect behavior.
#[derive(Debug, Default)]
pub struct FakeTransport {
    connected: bool,
    drop_next: bool,
    reorder_next: bool,
    delivered: Vec<EncodedFrame>,
}

#[allow(missing_docs)]
impl FakeTransport {
    pub fn connect(&mut self) {
        self.connected = true;
    }
    pub fn reconnect(&mut self) {
        self.connected = true;
        self.delivered.clear();
    }
    pub fn drop_next(&mut self) {
        self.drop_next = true;
    }
    pub fn reorder_next(&mut self) {
        self.reorder_next = true;
    }
    /// Sends a frame, dropping configured loss and reordering deterministically.
    ///
    /// # Errors
    /// Returns [`BackendError::Unavailable`] while disconnected.
    pub fn send(&mut self, frame: EncodedFrame) -> Result<(), BackendError> {
        if !self.connected {
            return Err(BackendError::Unavailable);
        }
        if self.drop_next {
            self.drop_next = false;
            return Ok(());
        }
        if self.reorder_next {
            self.reorder_next = false;
            if let Some(previous) = self.delivered.pop() {
                self.delivered.push(frame);
                self.delivered.push(previous);
                return Ok(());
            }
        }
        self.delivered.push(frame);
        Ok(())
    }
    #[must_use]
    pub fn delivered(&self) -> &[EncodedFrame] {
        &self.delivered
    }
}

/// Deterministic capture source for tests and architecture bring-up.
#[derive(Debug)]
pub struct FakeCapture {
    format: VideoFormat,
    remaining: u64,
    sequence: u64,
    fail_next: bool,
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
            fail_next: false,
        })
    }

    /// Makes the next frame request return a bounded I/O error.
    pub fn fail_next_frame(&mut self) {
        self.fail_next = true;
    }
}

impl CaptureBackend for FakeCapture {
    fn format(&self) -> VideoFormat {
        self.format
    }

    fn next_frame(&mut self) -> Result<CapturedFrame, BackendError> {
        if self.fail_next {
            self.fail_next = false;
            return Err(BackendError::DeviceIo);
        }
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
    fail_next: bool,
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

    /// Makes the next controller report return a bounded I/O error.
    pub fn fail_next_send(&mut self) {
        self.fail_next = true;
    }
}

impl ControllerBackend for FakeController {
    fn send_state(&mut self, state: ControllerState) -> Result<(), BackendError> {
        if self.fail_next {
            self.fail_next = false;
            return Err(BackendError::DeviceIo);
        }
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

    #[test]
    fn injected_failures_are_bounded_and_recoverable() {
        let mut capture = FakeCapture::new(format(), 1).unwrap();
        capture.fail_next_frame();
        assert_eq!(capture.next_frame(), Err(BackendError::DeviceIo));
        assert_eq!(capture.next_frame().unwrap().sequence, 0);
        let mut controller = FakeController::default();
        controller.fail_next_send();
        assert_eq!(
            controller.send_state(ControllerState::default()),
            Err(BackendError::DeviceIo)
        );
        controller.neutralize().unwrap();
        assert_eq!(controller.latest(), Some(ControllerState::default()));
    }

    #[test]
    fn fake_media_pipeline_bounds_loss_reorder_and_reconnect() {
        let mut queue = BoundedMediaQueue::new(NonZeroUsize::new(2).unwrap());
        queue.push(EncodedFrame {
            sequence: 1,
            bytes: vec![1],
        });
        queue.push(EncodedFrame {
            sequence: 2,
            bytes: vec![2],
        });
        queue.push(EncodedFrame {
            sequence: 3,
            bytes: vec![3],
        });
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.dropped(), 1);
        let mut transport = FakeTransport::default();
        transport.connect();
        transport.drop_next();
        transport.send(queue.pop().unwrap()).unwrap();
        transport.reorder_next();
        transport.send(queue.pop().unwrap()).unwrap();
        assert_eq!(transport.delivered().len(), 1);
        transport.reconnect();
        assert!(transport.delivered().is_empty());
    }

    #[test]
    fn v4l2_capabilities_fail_closed_before_allocation() {
        let requested = format();
        let capabilities = V4l2Capabilities {
            streaming: true,
            formats: vec![PixelFormat::Mjpeg],
        };
        assert_eq!(
            capabilities.negotiate(requested),
            Err(BackendError::Unavailable)
        );
        let capabilities = V4l2Capabilities {
            streaming: true,
            formats: vec![PixelFormat::Yuyv],
        };
        assert_eq!(capabilities.negotiate(requested), Ok(requested));
        let nv12 = VideoFormat {
            pixel_format: PixelFormat::Nv12,
            ..requested
        };
        let capabilities = V4l2Capabilities {
            streaming: true,
            formats: vec![PixelFormat::Yuyv, PixelFormat::Nv12],
        };
        assert_eq!(capabilities.negotiate(nv12), Ok(nv12));
        assert_eq!(requested.max_frame_bytes(), Ok(4_147_200));
        assert_eq!(nv12.max_frame_bytes(), Ok(3_110_400));
    }

    #[test]
    fn v4l2_mmap_configuration_is_bounded() {
        let config = V4l2MmapConfig {
            format: format(),
            buffer_count: NonZeroUsize::new(4).unwrap(),
        };
        assert_eq!(config.validate(), Ok(config));
        let oversized = V4l2MmapConfig {
            buffer_count: NonZeroUsize::new(9).unwrap(),
            ..config
        };
        assert_eq!(
            oversized.validate(),
            Err(BackendError::InvalidConfiguration)
        );
    }

    #[test]
    fn capture_timestamp_discontinuity_detects_regression_only() {
        let first = MonoTime::from_duration(Duration::from_millis(10));
        let later = MonoTime::from_duration(Duration::from_millis(20));
        assert!(!timestamp_discontinuity(None, first));
        assert!(!timestamp_discontinuity(Some(first), later));
        assert!(!timestamp_discontinuity(Some(first), first));
        assert!(timestamp_discontinuity(Some(later), first));
        let mut tracker = TimestampTracker::new();
        assert!(!tracker.observe(first));
        assert!(!tracker.observe(later));
        assert!(tracker.observe(first));
    }
}
