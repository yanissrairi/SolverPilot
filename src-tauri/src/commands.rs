use tauri::State;

use crate::config::AppConfig;
use crate::ssh::SshKeyStatus;
use crate::state::{AppState, Benchmark, Job, JobStatus, JobStatusResponse, SyncStatus};
use crate::{db, job, ssh};

// ============================================================================
// Configuration
// ============================================================================

#[tauri::command]
pub async fn load_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = AppConfig::load()?;

    // Initialiser la DB
    let db_path = AppConfig::db_path()?;
    let pool = db::init_db(&db_path.to_string_lossy()).await?;

    // Stocker dans l'état
    *state.config.lock().await = Some(config.clone());
    *state.db.lock().await = Some(pool);

    Ok(config)
}

// ============================================================================
// SSH
// ============================================================================

#[tauri::command]
pub async fn init_ssh(state: State<'_, AppState>) -> Result<String, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let socket = ssh::init_control_master(&config).await?;
    *state.ssh_socket.lock().await = Some(socket.clone());

    Ok(socket)
}

#[tauri::command]
pub async fn close_ssh(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().await.clone();
    if let Some(config) = config {
        ssh::close_control_master(&config).await?;
    }
    *state.ssh_socket.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn test_ssh(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    ssh::test_connection(&config).await
}

#[tauri::command]
pub async fn check_ssh_key_status(state: State<'_, AppState>) -> Result<SshKeyStatus, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    Ok(ssh::check_key_in_agent(&config).await)
}

#[tauri::command]
pub async fn add_ssh_key(state: State<'_, AppState>, passphrase: String) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    ssh::add_key_to_agent(&config, &passphrase).await
}

// ============================================================================
// Sync
// ============================================================================

#[tauri::command]
pub async fn check_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    match ssh::rsync_dry_run(&config).await {
        Ok(files) => {
            if files.is_empty() {
                Ok(SyncStatus::UpToDate)
            } else {
                Ok(SyncStatus::Modified {
                    count: files.len(),
                    files,
                })
            }
        }
        Err(e) => Ok(SyncStatus::Error { message: e }),
    }
}

#[tauri::command]
pub async fn sync_code(state: State<'_, AppState>) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    ssh::rsync_to_server(&config).await
}

// ============================================================================
// Benchmarks
// ============================================================================

#[tauri::command]
pub async fn scan_benchmarks(state: State<'_, AppState>) -> Result<Vec<Benchmark>, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let benchmarks = job::scan_benchmarks(&config.paths.local_benchmarks);
    Ok(benchmarks)
}

// ============================================================================
// Jobs
// ============================================================================

#[tauri::command]
pub async fn queue_jobs(
    state: State<'_, AppState>,
    benchmark_names: Vec<String>,
) -> Result<Vec<Job>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let mut jobs = Vec::new();
    for name in benchmark_names {
        let id = db::insert_job(&pool, &name).await?;
        jobs.push(Job {
            id,
            benchmark_name: name,
            status: JobStatus::Pending,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            finished_at: None,
            progress_current: 0,
            progress_total: 0,
            results_path: None,
            error_message: None,
            log_content: String::new(),
        });
    }

    Ok(jobs)
}

#[tauri::command]
pub async fn start_next_job(state: State<'_, AppState>) -> Result<Option<Job>, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    // Vérifier s'il y a déjà un job en cours
    if db::load_running_job(&pool).await?.is_some() {
        return Err("Un job est déjà en cours".to_string());
    }

    // Prendre le prochain job en attente
    let pending = db::load_pending_jobs(&pool).await?;
    if let Some(job) = pending.into_iter().next() {
        // Sync le code avant de lancer
        ssh::rsync_to_server(&config).await?;

        // Lancer le job
        ssh::start_tmux_job(&config, job.id, &job.benchmark_name).await?;

        // Mettre à jour le statut
        db::update_job_status(&pool, job.id, &JobStatus::Running).await?;

        // Stocker le job en cours
        *state.current_job_id.lock().await = Some(job.id);
        *state.job_start_time.lock().await = Some(std::time::Instant::now());

        // Recharger le job avec les nouvelles infos
        if let Some(running) = db::load_running_job(&pool).await? {
            return Ok(Some(running));
        }
    }

    Ok(None)
}

#[tauri::command]
pub async fn stop_job(state: State<'_, AppState>) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let job_id = *state.current_job_id.lock().await;
    if let Some(job_id) = job_id {
        ssh::stop_tmux_job(&config, job_id).await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn kill_job(state: State<'_, AppState>) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let job_id = *state.current_job_id.lock().await;
    if let Some(job_id) = job_id {
        ssh::kill_tmux_job(&config, job_id).await?;
        db::update_job_status(&pool, job_id, &JobStatus::Killed).await?;
        *state.current_job_id.lock().await = None;
        *state.job_start_time.lock().await = None;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_job_logs(state: State<'_, AppState>, lines: u32) -> Result<String, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let job_id = *state.current_job_id.lock().await;
    if let Some(job_id) = job_id {
        ssh::get_job_logs(&config, job_id, lines).await
    } else {
        Ok(String::new())
    }
}

#[tauri::command]
pub async fn get_job_status(state: State<'_, AppState>) -> Result<JobStatusResponse, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let job_id = *state.current_job_id.lock().await;
    let elapsed = state
        .job_start_time
        .lock()
        .await
        .as_ref()
        .map_or(0, |t| t.elapsed().as_secs());

    if let Some(job_id) = job_id {
        // Récupérer les logs
        let logs = ssh::get_job_logs(&config, job_id, 200)
            .await
            .unwrap_or_default();

        // Parser la progression
        let (current, total) = job::parse_progress(&logs).unwrap_or((0, 0));
        #[allow(clippy::cast_precision_loss)] // Précision suffisante pour un pourcentage
        let progress = if total > 0 {
            current as f32 / total as f32
        } else {
            0.0
        };
        let progress_text = if total > 0 {
            format!("[{current}/{total}]")
        } else {
            String::new()
        };

        // Détecter fin ou erreur
        let is_finished = job::detect_job_finished(&logs);
        let error = job::detect_job_error(&logs);

        // Vérifier si tmux existe encore
        let session_name = format!("job_{job_id}");
        let tmux_exists = ssh::tmux_session_exists(&config, &session_name)
            .await
            .unwrap_or(false);

        // Si le job est terminé ou tmux n'existe plus
        let job_done = is_finished || !tmux_exists;

        if job_done {
            // Mettre à jour le statut en DB
            let new_status = if error.is_some() {
                JobStatus::Failed
            } else {
                JobStatus::Completed
            };
            db::update_job_status(&pool, job_id, &new_status).await?;
            db::update_job_progress(&pool, job_id, current, total).await?;
            db::update_job_logs(&pool, job_id, &logs).await?;

            if let Some(ref err) = error {
                db::update_job_error(&pool, job_id, err).await?;
            }

            // Charger le job mis à jour
            let running_job = db::load_running_job(&pool).await?;

            return Ok(JobStatusResponse {
                job: running_job,
                logs,
                progress,
                progress_text,
                elapsed_seconds: elapsed,
                is_finished: true,
                error,
            });
        }

        // Mettre à jour la progression en DB
        db::update_job_progress(&pool, job_id, current, total).await?;

        // Charger le job actuel
        let running_job = db::load_running_job(&pool).await?;

        Ok(JobStatusResponse {
            job: running_job,
            logs,
            progress,
            progress_text,
            elapsed_seconds: elapsed,
            is_finished: false,
            error: None,
        })
    } else {
        Ok(JobStatusResponse {
            job: None,
            logs: String::new(),
            progress: 0.0,
            progress_text: String::new(),
            elapsed_seconds: 0,
            is_finished: false,
            error: None,
        })
    }
}

// ============================================================================
// History
// ============================================================================

#[tauri::command]
pub async fn load_history(state: State<'_, AppState>, limit: i32) -> Result<Vec<Job>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    db::load_history(&pool, limit).await
}

#[tauri::command]
pub async fn delete_job(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    db::delete_pending_job(&pool, job_id).await
}
