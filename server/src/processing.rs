use crate::{
    media,
    model::{App, Clip, Job, Progress},
};
use anyhow::{Context, bail};
use std::{
    collections::VecDeque,
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Stdio,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use zip::{ZipWriter, write::SimpleFileOptions};

struct SceneCut {
    time: f64,
    frame: u64,
}

pub async fn run(app: App, job: Job, threshold: f64) {
    let result = async {
        let _permit = app.worker.acquire().await?;
        app.update(job.id, Progress::Detecting { progress: 0.0 }, Some(vec![])).await;
        let cuts = detect(&app, &job, threshold).await?;
        app.update(job.id, Progress::Exporting { progress: 0.0 }, Some(cuts.iter().map(|cut| cut.time).collect())).await;
        let clips = export(&app, &job, &cuts).await?;
        app.update(job.id, Progress::Packaging, None).await;
        let dir = app.directory(job.id);
        let info = serde_json::json!({
            "source": job.name, "video": job.video, "clips": clips,
            "videoEncoding": "FFV1 lossless", "audioEncoding": "PCM 64-bit float, uncompressed",
            "notes": "Primary video and all audio tracks. Audio packets may straddle scene boundaries. Subtitles, attachments, and dynamic HDR metadata are not included."
        });
        tokio::fs::write(dir.join("manifest.json"), serde_json::to_vec_pretty(&info)?).await?;
        let archive_dir = dir.clone();
        let names = clips.iter().map(|clip| clip.name.clone()).collect::<Vec<_>>();
        let bytes = tokio::task::spawn_blocking(move || package(&archive_dir, names)).await??;
        app.update(job.id, Progress::Complete { clips, bytes }, None).await;
        anyhow::Ok(())
    }.await;
    if let Err(error) = result {
        eprintln!("Processing {} failed: {error:#}", job.id);
        app.update(
            job.id,
            Progress::Failed {
                message: error.to_string(),
            },
            None,
        )
        .await;
    }
}

async fn detect(app: &App, job: &Job, threshold: f64) -> anyhow::Result<Vec<SceneCut>> {
    let mut command = media::ffmpeg();
    command
        .arg("-i")
        .arg(app.directory(job.id).join("source"))
        .args(["-map", "0:V:0", "-an", "-sn", "-vf"])
        .arg(format!(
            "setpts=PTS-STARTPTS,select='if(gt(scene,{threshold}),print(n),0)',showinfo"
        ))
        .args(["-fps_mode", "vfr", "-progress", "pipe:2", "-f", "null", "-"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let mut lines =
        BufReader::new(child.stderr.take().context("Missing detection output")?).lines();
    let mut cuts = Vec::new();
    let mut recent = VecDeque::new();
    let mut last_progress = 0.0;
    let mut source_frame = None;
    while let Some(line) = lines.next_line().await? {
        // select prints the original frame number before showinfo prints its time.
        if line.starts_with("[Eval @") {
            source_frame = line
                .split_once("] ")
                .and_then(|(_, value)| value.split('.').next())
                .and_then(|value| value.parse::<u64>().ok());
        }
        if let Some(time) = line
            .split("pts_time:")
            .nth(1)
            .and_then(|v| v.split_whitespace().next())
            .and_then(|v| v.parse::<f64>().ok())
            && time > 0.0
            && cuts.last().is_none_or(|last: &SceneCut| time > last.time)
        {
            let frame = source_frame
                .take()
                .context("Could not read a scene's source frame number.")?;
            cuts.push(SceneCut { time, frame });
        }
        if let Some(time) = progress_time(&line) {
            let progress = (time / job.video.duration).clamp(0.0, 1.0);
            if progress - last_progress >= 0.01 {
                app.update(job.id, Progress::Detecting { progress }, None)
                    .await;
                last_progress = progress;
            }
        }
        if recent.len() == 12 {
            recent.pop_front();
        }
        recent.push_back(line);
    }
    if !child.wait().await?.success() {
        bail!(
            "Scene detection failed. {}",
            recent.into_iter().collect::<Vec<_>>().join("\n")
        );
    }
    Ok(cuts)
}

fn progress_time(line: &str) -> Option<f64> {
    line.strip_prefix("out_time_us=")?
        .parse::<f64>()
        .ok()
        .map(|value| value / 1_000_000.0)
}

async fn export(app: &App, job: &Job, cuts: &[SceneCut]) -> anyhow::Result<Vec<Clip>> {
    let dir = app.directory(job.id);
    let clips_dir = dir.join("clips");
    if tokio::fs::try_exists(&clips_dir).await? {
        tokio::fs::remove_dir_all(&clips_dir).await?;
    }
    tokio::fs::create_dir(&clips_dir).await?;
    let pixel_format = media::lossless_format(&job.video.pixel_format).await?;
    let mut command = media::ffmpeg();
    command
        .args(["-v", "warning"])
        .arg("-i")
        .arg(dir.join("source"))
        .args([
            "-map",
            "0:V:0",
            "-map",
            "0:a?",
            "-map_metadata",
            "0",
            "-c:v",
            "ffv1",
            "-level",
            "3",
            "-g",
            "1",
            "-slicecrc",
            "1",
            "-pix_fmt",
        ])
        .arg(pixel_format)
        .args([
            "-c:a",
            "pcm_f64le",
            "-fps_mode",
            "passthrough",
            "-enc_time_base:v",
            "demux",
            "-avoid_negative_ts",
            "make_zero",
            "-progress",
            "pipe:2",
            "-f",
            "segment",
            "-segment_format",
            "matroska",
            "-reset_timestamps",
            "1",
        ]);
    if job.video.pixel_format.starts_with("yuvj") {
        command.args([
            "-color_range",
            "pc",
            "-vf",
            "scale=in_range=full:out_range=full",
        ]);
    }
    command.args(["-threads", "2"]);
    // Frame indices avoid cut drift from Matroska's millisecond timestamp rounding.
    if cuts.is_empty() {
        command.args(["-segment_time", &(job.video.duration + 1.0).to_string()]);
    } else {
        command.arg("-segment_frames").arg(
            cuts.iter()
                .map(|cut| cut.frame.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    command
        .arg(clips_dir.join("scene_%04d.mkv"))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let mut lines = BufReader::new(child.stderr.take().context("Missing export output")?).lines();
    let mut recent = VecDeque::new();
    let mut last_progress = 0.0;
    while let Some(line) = lines.next_line().await? {
        if let Some(time) = progress_time(&line) {
            let progress = (time / job.video.duration).clamp(0.0, 1.0);
            if progress - last_progress >= 0.01 {
                app.update(job.id, Progress::Exporting { progress }, None)
                    .await;
                last_progress = progress;
            }
        }
        if recent.len() == 12 {
            recent.pop_front();
        }
        recent.push_back(line);
    }
    if !child.wait().await?.success() {
        bail!(
            "Lossless export failed. {}",
            recent.into_iter().collect::<Vec<_>>().join("\n")
        );
    }
    let boundaries: Vec<f64> = std::iter::once(0.0)
        .chain(cuts.iter().map(|cut| cut.time))
        .chain(std::iter::once(job.video.duration))
        .collect();
    let mut clips = Vec::new();
    for (index, range) in boundaries.windows(2).enumerate() {
        let name = format!("scene_{index:04}.mkv");
        let bytes = tokio::fs::metadata(clips_dir.join(&name))
            .await
            .context("An expected scene clip is missing")?
            .len();
        clips.push(Clip {
            name,
            start: range[0],
            end: range[1],
            bytes,
        });
    }
    Ok(clips)
}

fn package(dir: &Path, names: Vec<String>) -> anyhow::Result<u64> {
    let file = File::create(dir.join("scenes.zip.tmp"))?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .large_file(true);
    let mut buffer = vec![0; 1024 * 1024];
    for name in names
        .into_iter()
        .chain(std::iter::once("manifest.json".into()))
    {
        let path = if name == "manifest.json" {
            dir.join(&name)
        } else {
            dir.join("clips").join(&name)
        };
        let mut input = File::open(path)?;
        archive.start_file(name, options)?;
        loop {
            let bytes = input.read(&mut buffer)?;
            if bytes == 0 {
                break;
            }
            archive.write_all(&buffer[..bytes])?;
        }
    }
    archive.finish()?.sync_all()?;
    std::fs::rename(dir.join("scenes.zip.tmp"), dir.join("scenes.zip"))?;
    Ok(std::fs::metadata(dir.join("scenes.zip"))?.len())
}
