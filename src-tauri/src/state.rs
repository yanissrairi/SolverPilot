use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::AppConfig;

/// État global de l'application (thread-safe)
pub struct AppState {
    pub config: Arc<Mutex<Option<AppConfig>>>,
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub ssh_socket: Arc<Mutex<Option<String>>>,
    pub current_job_id: Arc<Mutex<Option<i64>>>,
    pub job_start_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Arc::new(Mutex::new(None)),
            db: Arc::new(Mutex::new(None)),
            ssh_socket: Arc::new(Mutex::new(None)),
            current_job_id: Arc::new(Mutex::new(None)),
            job_start_time: Arc::new(Mutex::new(None)),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Un benchmark Python disponible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub name: String,
    pub path: String,
}

/// État d'un job
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Killed,
}

/// Un job de benchmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub benchmark_name: String,
    pub status: JobStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub progress_current: u32,
    pub progress_total: u32,
    pub results_path: Option<String>,
    pub error_message: Option<String>,
    pub log_content: String,
}

/// Status de synchronisation du code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SyncStatus {
    Checking,
    UpToDate,
    Modified { count: usize, files: Vec<String> },
    Syncing,
    Error { message: String },
}

/// Réponse du status d'un job en cours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub job: Option<Job>,
    pub logs: String,
    pub progress: f32,
    pub progress_text: String,
    pub elapsed_seconds: u64,
    pub is_finished: bool,
    pub error: Option<String>,
}
