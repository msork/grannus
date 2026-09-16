//! Grannus host composition root.

#![forbid(unsafe_code)]

mod config;
mod signal;

use config::Config;
use grannus_core::{
    Button, Buttons, ClientIdentity, ControllerState, Lifecycle, LifecycleState, MetricEvent,
    MetricEventBuffer, MetricStage, MonoClock, PlayerSlot, SequenceDecision, SequenceWindow,
    SlotManager, TraceId,
};
use grannus_platform::{
    CaptureBackend, ControllerBackend, FakeCapture, FakeController, PixelFormat, VideoFormat,
};
use grannus_protocol::{InputDatagram, decode_input_v1, encode_input_v1};
use std::env;
use std::error::Error;
use std::num::NonZeroUsize;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("grannus: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    match arguments.next().as_deref() {
        Some("doctor") => {
            let config = load_config(arguments)?;
            doctor(&config);
            Ok(())
        }
        Some("fake-session") => {
            load_config(arguments)?;
            fake_session(&signal::SignalShutdown::install()?)
        }
        Some("help" | "--help" | "-h") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown command {other:?}; run with --help").into()),
    }
}

fn load_config(mut arguments: impl Iterator<Item = String>) -> Result<Config, Box<dyn Error>> {
    match arguments.next().as_deref() {
        None => Ok(Config::default()),
        Some("--config") => {
            let path = arguments.next().ok_or("--config requires a path")?;
            if arguments.next().is_some() {
                return Err("unexpected argument after configuration path".into());
            }
            Ok(Config::load(path)?)
        }
        Some(other) => {
            Err(format!("unexpected argument {other:?}; expected --config <PATH>").into())
        }
    }
}

fn doctor(config: &Config) {
    println!("Grannus {}", env!("CARGO_PKG_VERSION"));
    println!("target_os={}", env::consts::OS);
    println!("target_arch={}", env::consts::ARCH);
    println!("hardware_backends=not-built");
    println!("network_listeners=none");
    println!("config_listen={}", config.host.listen);
    println!("status=foundation-only");
}

fn fake_session(shutdown: &signal::SignalShutdown) -> Result<(), Box<dyn Error>> {
    let mut lifecycle = Lifecycle::new();
    let clock = MonoClock::new();
    let mut metric_events = MetricEventBuffer::new(NonZeroUsize::new(16).unwrap());
    let format = VideoFormat {
        width: 1_920,
        height: 1_080,
        fps_numerator: 60,
        fps_denominator: 1,
        pixel_format: PixelFormat::Yuyv,
    };
    let mut capture = FakeCapture::new(format, 3)?;
    let mut controller = FakeController::default();
    let mut frame_count = 0_u64;
    while !shutdown.requested() {
        let frame = match capture.next_frame() {
            Ok(frame) => frame,
            Err(grannus_platform::BackendError::EndOfStream) => break,
            Err(error) => {
                lifecycle.request_shutdown();
                controller.neutralize()?;
                lifecycle.complete_shutdown();
                return Err(error.into());
            }
        };
        metric_events.record(MetricEvent::new(
            frame.trace_id,
            MetricStage::CaptureArrival,
            frame.captured_at,
        ));
        frame_count += 1;
    }

    if shutdown.requested() {
        lifecycle.request_shutdown();
        controller.neutralize()?;
        lifecycle.complete_shutdown();
        println!("fake_session_frames={frame_count}");
        println!("fake_controller_reports={}", controller.reports());
        println!("final_state_neutral=true");
        println!("status=shutdown");
        return Ok(());
    }

    let state = ControllerState {
        buttons: Buttons::default().with(Button::South, true),
        ..ControllerState::default()
    };
    let input = InputDatagram {
        sequence: 1,
        source_time_us: 100,
        player_slot: PlayerSlot::new(0)?,
        state,
    };
    let decoded = decode_input_v1(&encode_input_v1(&input))?;
    let mut sequences = SequenceWindow::new();
    if sequences.observe(decoded.sequence) != SequenceDecision::Accepted {
        return Err("initial fake input sequence was not accepted".into());
    }
    let input_identity = ClientIdentity::new(1);
    let mut slots = SlotManager::new();
    slots.claim(decoded.player_slot, input_identity)?;
    slots.authorize(decoded.player_slot, input_identity)?;
    let input_trace = TraceId::new(u64::MAX);
    metric_events.record(MetricEvent::new(
        input_trace,
        MetricStage::InputAccepted,
        clock.now(),
    ));
    if let Err(error) = controller.send_state(decoded.state) {
        lifecycle.request_shutdown();
        controller.neutralize()?;
        lifecycle.complete_shutdown();
        return Err(error.into());
    }
    metric_events.record(MetricEvent::new(
        input_trace,
        MetricStage::ControllerReport,
        clock.now(),
    ));
    lifecycle.request_shutdown();
    controller.neutralize()?;
    lifecycle.complete_shutdown();
    metric_events.record(MetricEvent::new(
        input_trace,
        MetricStage::ControllerReport,
        clock.now(),
    ));

    println!("fake_session_frames={frame_count}");
    println!("fake_controller_reports={}", controller.reports());
    println!("input_slot_authorized=true");
    println!("input_sequence_accepted=true");
    println!("metric_events={}", metric_events.events().len());
    println!("metric_events_dropped={}", metric_events.dropped_events());
    println!(
        "final_state_neutral={}",
        controller.latest() == Some(ControllerState::default())
    );
    println!("status=ok");
    debug_assert_eq!(lifecycle.state(), LifecycleState::Stopped);
    Ok(())
}

fn print_help() {
    println!("Grannus host (pre-alpha)");
    println!();
    println!("USAGE:");
    println!("    grannus-host <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!(
        "    doctor [--config PATH]        Validate config and report safe foundation/backend status"
    );
    println!(
        "    fake-session [--config PATH]  Validate config and exercise deterministic fake paths"
    );
    println!("    help          Show this help");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_commands_succeed() {
        assert!(run(["doctor".to_owned()].into_iter()).is_ok());
        assert!(run(["fake-session".to_owned()].into_iter()).is_ok());
    }

    #[test]
    fn unknown_command_fails() {
        assert!(run(["serve".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn configuration_flag_requires_a_path() {
        assert!(run(["doctor".to_owned(), "--config".to_owned()].into_iter()).is_err());
    }

    #[test]
    fn fault_cleanup_neutralizes_controller_before_stopping() {
        let mut lifecycle = Lifecycle::new();
        let mut controller = FakeController::default();
        controller.fail_next_send();
        let error = controller
            .send_state(ControllerState::default())
            .unwrap_err();
        lifecycle.request_shutdown();
        controller.neutralize().unwrap();
        lifecycle.complete_shutdown();
        assert_eq!(error, grannus_platform::BackendError::DeviceIo);
        assert_eq!(controller.latest(), Some(ControllerState::default()));
        assert_eq!(lifecycle.state(), LifecycleState::Stopped);
    }
}
