use crate::{
    media,
    model::{App, Job, Progress},
    processing,
};
use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, State},
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use uuid::Uuid;

pub struct ApiError(pub StatusCode, pub String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({"error": self.1}))).into_response()
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        eprintln!("API error: {error:#}");
        Self(StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
    }
}

pub async fn upload(
    State(app): State<App>,
    mut multipart: Multipart,
) -> Result<Json<Job>, ApiError> {
    let id = Uuid::new_v4();
    let dir = app.directory(id);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(anyhow::Error::from)?;
    let result = async {
        let mut field = multipart
            .next_field()
            .await
            .map_err(|e| anyhow::anyhow!("Upload interrupted: {e}"))?
            .ok_or_else(|| anyhow::anyhow!("Choose a video to upload."))?;
        if field.name() != Some("video") {
            anyhow::bail!("Expected a video file.");
        }
        let name = field
            .file_name()
            .and_then(|n| std::path::Path::new(n).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("video")
            .to_owned();
        let mut file = tokio::fs::File::create(dir.join("source")).await?;
        let mut bytes = 0_u64;
        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk)
                .await
                .context("Could not save upload. Check available disk space.")?;
            bytes += chunk.len() as u64;
        }
        file.flush().await?;
        drop(file);
        if bytes == 0 {
            anyhow::bail!("This file is empty. Choose a video.");
        }
        let video = media::probe(&dir.join("source"), bytes).await?;
        let job = Job {
            id,
            name,
            video,
            cuts: vec![],
            status: Progress::Ready,
        };
        // Preview is disposable; it is never used to create lossless clips.
        media::poster(&dir.join("source"), &dir.join("poster.jpg")).await;
        app.save(&job).await?;
        app.jobs.write().await.insert(id, job.clone());
        anyhow::Ok(job)
    }
    .await;
    match result {
        Ok(job) => Ok(Json(job)),
        Err(error) => {
            if let Err(cleanup) = tokio::fs::remove_dir_all(&dir).await {
                eprintln!("Upload cleanup: {cleanup}");
            }
            Err(error.into())
        }
    }
}

use anyhow::Context;

pub async fn get(State(app): State<App>, Path(id): Path<Uuid>) -> Result<Json<Job>, ApiError> {
    Ok(Json(app.get(id).await.ok_or_else(not_found)?))
}

fn not_found() -> ApiError {
    ApiError(
        StatusCode::NOT_FOUND,
        "This upload no longer exists. Upload the video again.".into(),
    )
}

#[derive(Deserialize)]
pub struct Split {
    threshold: f64,
}

pub async fn split(
    State(app): State<App>,
    Path(id): Path<Uuid>,
    Json(options): Json<Split>,
) -> Result<Json<Job>, ApiError> {
    if !options.threshold.is_finite() || !(0.01..=0.99).contains(&options.threshold) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Scene threshold must be between 0.01 and 0.99.".into(),
        ));
    }
    let mut jobs = app.jobs.write().await;
    let mut job = jobs.get(&id).ok_or_else(not_found)?.clone();
    if job.status.active() {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "This video is already processing.".into(),
        ));
    }
    job.status = Progress::Queued;
    job.cuts.clear();
    app.save(&job).await?;
    jobs.insert(id, job.clone());
    drop(jobs);
    let worker_app = app.clone();
    let worker_job = job.clone();
    tokio::spawn(async move {
        processing::run(worker_app, worker_job, options.threshold).await;
    });
    Ok(Json(job))
}

pub async fn delete(State(app): State<App>, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    let mut jobs = app.jobs.write().await;
    let job = jobs.get(&id).ok_or_else(not_found)?;
    if job.status.active() {
        return Err(ApiError(
            StatusCode::CONFLICT,
            "Wait for processing to finish before deleting this upload.".into(),
        ));
    }
    tokio::fs::remove_dir_all(app.directory(id))
        .await
        .map_err(anyhow::Error::from)?;
    jobs.remove(&id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn asset(
    State(app): State<App>,
    Path((id, asset)): Path<(Uuid, String)>,
    request: Request<Body>,
) -> Result<Response, ApiError> {
    let job = app.get(id).await.ok_or_else(not_found)?;
    let (filename, content_type) = match asset.as_str() {
        "source" => {
            let container = &job.video.container;
            let mime = if container.contains("mp4") || container.contains("mov") {
                "video/mp4"
            } else if container.contains("webm") && job.name.to_ascii_lowercase().ends_with(".webm")
            {
                "video/webm"
            } else if container.contains("matroska") {
                "video/x-matroska"
            } else {
                "application/octet-stream"
            };
            ("source", Some(mime))
        }
        "poster" => ("poster.jpg", Some("image/jpeg")),
        "archive" if matches!(job.status, Progress::Complete { .. }) => {
            ("scenes.zip", Some("application/zip"))
        }
        _ => return Err(not_found()),
    };
    let response = ServeFile::new(app.directory(id).join(filename))
        .oneshot(request)
        .await
        .map_err(|error| anyhow::anyhow!("Could not read file: {error}"))?;
    let mut response = response.into_response();
    if let Some(content_type) = content_type {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            content_type.parse().expect("static header"),
        );
    }
    if asset == "archive" {
        let stem = std::path::Path::new(&job.name)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("video");
        let name = format!("{stem}.zip");
        let fallback: String = name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || " .-_".contains(character) {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let disposition = format!(
            "attachment; filename=\"{fallback}\"; filename*=UTF-8''{}",
            utf8_percent_encode(&name, NON_ALPHANUMERIC)
        );
        response.headers_mut().insert(
            header::CONTENT_DISPOSITION,
            disposition.parse().map_err(anyhow::Error::from)?,
        );
    }
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        "private, no-store".parse().expect("static header"),
    );
    Ok(response)
}
