use anyhow::{anyhow, bail, Context, Result};
use openrazer_adapter::OpenrazerAdapter;
use opensynapse_core::{Color, DeviceId, LightingAdapter, LightingMode, LightingSpec};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        bail!(usage());
    }

    let vid = parse_hex_u16(&args[0]).context("vid")?;
    let pid = parse_hex_u16(&args[1]).context("pid")?;
    let mode = parse_mode(&args[2])?;
    let color = args.get(3).map(|s| parse_color(s)).transpose()?;
    let brightness = args.get(4).map(|s| s.parse::<u8>()).transpose()?;

    let spec = LightingSpec {
        mode,
        color,
        brightness,
    };
    let id = DeviceId {
        vid,
        pid,
        serial: None,
    };

    let adapter = OpenrazerAdapter::connect().await?;
    adapter.apply_lighting(&id, &spec).await?;

    println!("ok: applied {:?} to {:04x}:{:04x}", spec.mode, vid, pid);
    Ok(())
}

fn usage() -> String {
    "usage: openrazer-apply <vid_hex> <pid_hex> <mode> [color_rrggbb] [brightness 0..100]\n\
       modes: none | static | breathing | spectrum | wave | reactive"
        .into()
}

fn parse_hex_u16(s: &str) -> Result<u16> {
    let s = s.trim_start_matches("0x");
    u16::from_str_radix(s, 16).map_err(|e| anyhow!("hex u16 parse error: {e}"))
}

fn parse_mode(s: &str) -> Result<LightingMode> {
    Ok(match s.to_ascii_lowercase().as_str() {
        "none" => LightingMode::None,
        "static" => LightingMode::Static,
        "breathing" => LightingMode::Breathing,
        "spectrum" => LightingMode::Spectrum,
        "wave" => LightingMode::Wave,
        "reactive" => LightingMode::Reactive,
        other => bail!("unknown mode: {other}\n{}", usage()),
    })
}

fn parse_color(s: &str) -> Result<Color> {
    if s.len() != 6 {
        bail!("color must be 6 hex digits (RRGGBB), got {s:?}");
    }
    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;
    Ok(Color { r, g, b })
}
