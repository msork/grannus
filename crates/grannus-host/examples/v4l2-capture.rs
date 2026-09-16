#![allow(missing_docs)]

use grannus_platform::{PixelFormat, capture_v4l2_mmap_once};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let value = |name: &str| -> Result<String, Box<dyn std::error::Error>> {
        args.windows(2)
            .find(|pair| pair[0] == name)
            .map(|pair| pair[1].clone())
            .ok_or_else(|| format!("missing {name}").into())
    };
    let device = value("--device")?;
    let width: u16 = value("--width")?.parse()?;
    let height: u16 = value("--height")?.parse()?;
    let format = match value("--pixel-format")?.as_str() {
        "YUYV" => PixelFormat::Yuyv,
        "NV12" => PixelFormat::Nv12,
        other => return Err(format!("unsupported pixel format {other}").into()),
    };
    let frames: u32 = value("--frames")?.parse()?;
    if frames == 0 || frames > 10_000 {
        return Err("frames must be 1..=10000".into());
    }
    for index in 0..frames {
        let frame = capture_v4l2_mmap_once(&device, width, height, format)
            .map_err(|error| format!("capture {index}: {error}"))?;
        println!(
            "frame={} sequence={} bytes={} timestamp_us={}",
            index,
            frame.sequence,
            frame.bytes.len(),
            frame.captured_at.duration().as_micros()
        );
    }
    println!("frames={frames} status=ok");
    Ok(())
}
