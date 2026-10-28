use crate::{model::Video, toolchain};
use anyhow::{Context, bail};
use serde::Deserialize;
use std::{path::Path, process::Stdio};
use tokio::process::Command;

#[derive(Deserialize)]
struct Probe {
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Deserialize)]
struct Stream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    pix_fmt: Option<String>,
    avg_frame_rate: Option<String>,
    duration: Option<String>,
    #[serde(default)]
    disposition: Disposition,
}

#[derive(Default, Deserialize)]
struct Disposition {
    #[serde(default)]
    attached_pic: u8,
}

#[derive(Deserialize)]
struct Format {
    duration: Option<String>,
    format_name: String,
}

pub fn ffmpeg() -> Command {
    let mut command = toolchain::ffmpeg();
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-y",
            "-threads",
            "2",
            "-protocol_whitelist",
            "file,pipe",
        ])
        .stdin(Stdio::null())
        .kill_on_drop(true);
    command
}

pub async fn probe(path: &Path, bytes: u64) -> anyhow::Result<Video> {
    let output = toolchain::ffprobe()
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(path)
        .kill_on_drop(true)
        .output()
        .await
        .context("Could not run ffprobe")?;
    if !output.stderr.is_empty() {
        let diagnostics = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "ffprobe diagnostics: {}",
            diagnostics.chars().take(4000).collect::<String>()
        );
    }
    if !output.status.success() {
        bail!(
            "This file could not be decoded as a video. It may be damaged or use an unsupported codec."
        );
    }
    let probe: Probe = serde_json::from_slice(&output.stdout)?;
    let video = probe
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video") && s.disposition.attached_pic == 0)
        .context("No playable video track found in this file.")?;
    let duration = video
        .duration
        .as_ref()
        .or(probe.format.duration.as_ref())
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|n| n.is_finite() && *n > 0.0)
        .context("This video has no readable duration.")?;
    let fps = video
        .avg_frame_rate
        .as_deref()
        .and_then(|v| v.split_once('/'))
        .and_then(|(a, b)| Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?))
        .filter(|v| v.is_finite() && *v > 0.0)
        .unwrap_or(30.0);
    Ok(Video {
        duration,
        width: video.width.context("Video width is missing.")?,
        height: video.height.context("Video height is missing.")?,
        fps,
        codec: video.codec_name.clone().unwrap_or_else(|| "unknown".into()),
        pixel_format: video.pix_fmt.clone().filter(|format| format != "unknown" && !format.is_empty())
            .with_context(|| format!(
                "Could not determine pixel format for {} video. The server may lack its decoder, or the file may be damaged. No quality-reducing conversion was made.",
                video.codec_name.as_deref().unwrap_or("this")
            ))?,
        container: probe.format.format_name,
        audio_tracks: probe
            .streams
            .iter()
            .filter(|s| s.codec_type.as_deref() == Some("audio"))
            .count(),
        bytes,
    })
}

// A forced source pixel format prevents FFmpeg from silently reducing precision.
// RGB packing conversions change storage layout, not pixel values.
pub async fn lossless_format(source: &str) -> anyhow::Result<String> {
    let output = ffmpeg().args(["-h", "encoder=ffv1"]).output().await?;
    let help = String::from_utf8_lossy(&output.stdout);
    let supported = help
        .lines()
        .find_map(|line| line.trim().strip_prefix("Supported pixel formats:"))
        .context("Could not read FFV1 capabilities.")?;
    let target = match source {
        "rgb24" | "bgr24" | "gbrp" => "bgr0",
        "rgba" | "argb" | "abgr" | "gbrap" => "bgra",
        "yuvj420p" => "yuv420p",
        "yuvj422p" => "yuv422p",
        "yuvj444p" => "yuv444p",
        "yuvj440p" => "yuv440p",
        value => value,
    };
    if !supported.split_whitespace().any(|format| format == target) {
        bail!(
            "Pixel format {source} cannot be exported losslessly by this FFmpeg build. No quality-reducing conversion was made."
        );
    }
    Ok(target.to_owned())
}

pub async fn poster(source: &Path, destination: &Path) {
    let result = ffmpeg()
        .args(["-v", "error"])
        .arg("-i")
        .arg(source)
        .args([
            "-map",
            "0:V:0",
            "-frames:v",
            "1",
            "-vf",
            "scale=960:-2",
            "-update",
            "1",
        ])
        .arg(destination)
        .output()
        .await;
    if let Err(error) = result {
        eprintln!("Poster generation failed: {error}");
    }
}
