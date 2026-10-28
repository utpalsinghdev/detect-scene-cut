use anyhow::{Context, ensure};
use std::{path::PathBuf, sync::LazyLock};
use tokio::process::Command;

struct MediaTools {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}

static TOOLS: LazyLock<MediaTools> = LazyLock::new(|| MediaTools {
    ffmpeg: resolve("FFMPEG_BIN", "ffmpeg"),
    ffprobe: resolve("FFPROBE_BIN", "ffprobe"),
});

fn resolve(variable: &str, program: &str) -> PathBuf {
    if let Some(path) = std::env::var_os(variable) {
        return PathBuf::from(path);
    }
    let local = PathBuf::from(".tools/ffmpeg/bin").join(program);
    if local.is_file() {
        local
    } else {
        PathBuf::from(program)
    }
}

pub fn ffmpeg() -> Command {
    Command::new(&TOOLS.ffmpeg)
}

pub fn ffprobe() -> Command {
    Command::new(&TOOLS.ffprobe)
}

async fn capabilities(command: &mut Command, option: &str) -> anyhow::Result<String> {
    let output = command.args(["-v", "error", option]).output().await
        .context("Media tools unavailable. Run bash scripts/install-ffmpeg.sh to install a matched build.")?;
    ensure!(
        output.status.success(),
        "Could not inspect media capabilities. Run bash scripts/install-ffmpeg.sh."
    );
    Ok(String::from_utf8(output.stdout)?)
}

fn contains(list: &str, name: &str) -> bool {
    list.lines()
        .any(|line| line.split_whitespace().nth(1) == Some(name))
}

pub async fn verify() -> anyhow::Result<()> {
    for (name, mut command) in [("FFmpeg", ffmpeg()), ("ffprobe", ffprobe())] {
        let decoders = capabilities(&mut command, "-decoders").await?;
        for codec in ["h264", "hevc"] {
            ensure!(
                contains(&decoders, codec),
                "{name} lacks full {codec} decoding. Run bash scripts/install-ffmpeg.sh. Do not substitute a lower-quality pixel format."
            );
        }
    }
    let encoders = capabilities(&mut ffmpeg(), "-encoders").await?;
    for codec in ["ffv1", "pcm_f64le"] {
        ensure!(
            contains(&encoders, codec),
            "FFmpeg is missing required encoder {codec}."
        );
    }
    let filters = capabilities(&mut ffmpeg(), "-filters").await?;
    for filter in ["select", "showinfo", "setpts", "scale"] {
        ensure!(
            contains(&filters, filter),
            "FFmpeg is missing required filter {filter}."
        );
    }
    Ok(())
}
