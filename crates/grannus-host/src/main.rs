//! Grannus host composition root.

#![forbid(unsafe_code)]

use grannus_core::{Button, Buttons, ControllerState, PlayerSlot};
use grannus_platform::{
    CaptureBackend, ControllerBackend, FakeCapture, FakeController, PixelFormat, VideoFormat,
};
use grannus_protocol::{InputDatagram, decode_input_v1, encode_input_v1};
use std::env;
use std::error::Error;
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
            doctor();
            Ok(())
        }
        Some("fake-session") => fake_session(),
        Some("help" | "--help" | "-h") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("unknown command {other:?}; run with --help").into()),
    }
}

fn doctor() {
    println!("Grannus {}", env!("CARGO_PKG_VERSION"));
    println!("target_os={}", env::consts::OS);
    println!("target_arch={}", env::consts::ARCH);
    println!("hardware_backends=not-built");
    println!("network_listeners=none");
    println!("status=foundation-only");
}

fn fake_session() -> Result<(), Box<dyn Error>> {
    let format = VideoFormat {
        width: 1_920,
        height: 1_080,
        fps_numerator: 60,
        fps_denominator: 1,
        pixel_format: PixelFormat::Yuyv,
    };
    let mut capture = FakeCapture::new(format, 3)?;
    let mut frame_count = 0_u64;
    while capture.next_frame().is_ok() {
        frame_count += 1;
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
    let mut controller = FakeController::default();
    controller.send_state(decoded.state)?;
    controller.neutralize()?;

    println!("fake_session_frames={frame_count}");
    println!("fake_controller_reports={}", controller.reports());
    println!("final_state_neutral={}", controller.latest() == Some(ControllerState::default()));
    println!("status=ok");
    Ok(())
}

fn print_help() {
    println!("Grannus host (pre-alpha)");
    println!();
    println!("USAGE:");
    println!("    grannus-host <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    doctor        Report safe foundation/backend status");
    println!("    fake-session  Exercise deterministic fake capture and input paths");
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
}
