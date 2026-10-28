use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

#[derive(Clone)]
pub struct App {
    pub jobs: Arc<RwLock<HashMap<Uuid, Job>>>,
    pub root: PathBuf,
    pub worker: Arc<Semaphore>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
    pub pixel_format: String,
    #[serde(default)]
    pub container: String,
    pub audio_tracks: usize,
    pub bytes: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub name: String,
    pub start: f64,
    pub end: f64,
    pub bytes: u64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "camelCase")]
pub enum Progress {
    Ready,
    Queued,
    Detecting { progress: f64 },
    Exporting { progress: f64 },
    Packaging,
    Complete { clips: Vec<Clip>, bytes: u64 },
    Failed { message: String },
}

impl Progress {
    pub fn active(&self) -> bool {
        matches!(
            self,
            Self::Queued | Self::Detecting { .. } | Self::Exporting { .. } | Self::Packaging
        )
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub video: Video,
    pub cuts: Vec<f64>,
    pub status: Progress,
}

impl App {
    pub fn directory(&self, id: Uuid) -> PathBuf {
        self.root.join(id.to_string())
    }

    pub async fn get(&self, id: Uuid) -> Option<Job> {
        self.jobs.read().await.get(&id).cloned()
    }

    pub async fn update(&self, id: Uuid, status: Progress, cuts: Option<Vec<f64>>) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&id) {
            job.status = status;
            if let Some(cuts) = cuts {
                job.cuts = cuts;
            }
            if let Err(error) = self.save(job).await {
                eprintln!("Could not persist job {id}: {error:#}");
            }
        }
    }

    pub async fn save(&self, job: &Job) -> anyhow::Result<()> {
        let dir = self.directory(job.id);
        tokio::fs::write(dir.join("job.json.tmp"), serde_json::to_vec(job)?).await?;
        tokio::fs::rename(dir.join("job.json.tmp"), dir.join("job.json")).await?;
        Ok(())
    }

    pub async fn restore(&self) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        let mut dirs = tokio::fs::read_dir(&self.root).await?;
        while let Some(dir) = dirs.next_entry().await? {
            let Ok(data) = tokio::fs::read(dir.path().join("job.json")).await else {
                continue;
            };
            let Ok(mut job) = serde_json::from_slice::<Job>(&data) else {
                continue;
            };
            if job.status.active() {
                job.status = Progress::Failed {
                    message: "Server restarted during processing. Split again to retry.".into(),
                };
                self.save(&job).await?;
            }
            self.jobs.write().await.insert(job.id, job);
        }
        Ok(())
    }
}
