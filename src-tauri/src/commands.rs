use std::path::Path;
use tauri::State;

use crate::config::AppConfig;
use crate::ssh::SshKeyStatus;
use crate::state::{AppState, Benchmark, Job, JobStatus, JobStatusResponse, Project, SyncStatus};
use crate::{db, job, project, python_deps, ssh};

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

    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    // Récupérer le projet actif
    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif - sélectionnez un projet d'abord")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    let project_dir = project::project_path(&proj.name)?;

    match ssh::rsync_dry_run(&config, &proj.name, &project_dir).await {
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

    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    // Récupérer le projet actif
    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif - sélectionnez un projet d'abord")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    let project_dir = project::project_path(&proj.name)?;

    ssh::rsync_project_to_server(&config, &proj.name, &project_dir).await
}

/// Synchronise uniquement les fichiers nécessaires pour un benchmark
/// (analyse les dépendances et sync les fichiers identifiés)
#[tauri::command]
pub async fn sync_benchmark_deps(
    state: State<'_, AppState>,
    benchmark_path: String,
) -> Result<usize, String> {
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    let benchmark_path = Path::new(&benchmark_path);

    // Le dossier parent du benchmark comme racine pour les imports locaux
    let local_code_root = benchmark_path
        .parent()
        .ok_or("Impossible de déterminer le dossier parent")?;

    // pyproject.toml du projet
    let pyproject_path = project::pyproject_path(&proj.name)?;
    let pyproject = if pyproject_path.exists() {
        Some(pyproject_path.as_path())
    } else {
        None
    };

    // Analyser les dépendances
    let mut analyzer = python_deps::PythonAnalyzer::new()?;
    let analysis = analyzer.analyze(benchmark_path, local_code_root, pyproject)?;

    // Collecter tous les fichiers à synchroniser
    let files = analysis.collect_all_file_paths();
    let file_count = files.len();

    // D'abord sync le projet (pyproject.toml, uv.lock)
    let project_dir = project::project_path(&proj.name)?;
    ssh::rsync_project_to_server(&config, &proj.name, &project_dir).await?;

    // Puis sync les fichiers du benchmark
    ssh::rsync_benchmark_files(&config, &proj.name, benchmark_path, &files).await?;

    Ok(file_count)
}

// ============================================================================
// Projects
// ============================================================================

/// Liste tous les projets
#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    db::list_projects(&pool).await
}

/// Crée un nouveau projet avec environnement uv
#[tauri::command]
pub async fn create_project(
    state: State<'_, AppState>,
    name: String,
    python_version: String,
) -> Result<Project, String> {
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

    // Créer le dossier et initialiser uv
    project::create_project(&name, &python_version, &config.tools.uv_path).await?;

    // Insérer en DB
    let id = db::insert_project(&pool, &name, &python_version).await?;

    db::get_project(&pool, id)
        .await?
        .ok_or_else(|| "Projet non trouvé après création".to_string())
}

/// Supprime un projet et son dossier
#[tauri::command]
pub async fn delete_project(state: State<'_, AppState>, project_id: i64) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    // Récupérer le nom pour supprimer le dossier
    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    // Supprimer de la DB (cascade sur benchmarks)
    db::delete_project(&pool, project_id).await?;

    // Supprimer le dossier
    project::delete_project_dir(&proj.name)?;

    // Si c'était le projet actif, le désélectionner
    let mut current = state.current_project_id.lock().await;
    if *current == Some(project_id) {
        *current = None;
    }
    drop(current);

    Ok(())
}

/// Définit le projet actif
#[tauri::command]
pub async fn set_active_project(
    state: State<'_, AppState>,
    project_id: i64,
) -> Result<Project, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    *state.current_project_id.lock().await = Some(project_id);

    Ok(proj)
}

/// Récupère le projet actif
#[tauri::command]
pub async fn get_active_project(state: State<'_, AppState>) -> Result<Option<Project>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let project_id = *state.current_project_id.lock().await;

    match project_id {
        Some(id) => db::get_project(&pool, id).await,
        None => Ok(None),
    }
}

// ============================================================================
// Python Version Management
// ============================================================================

/// Liste les versions Python disponibles
#[tauri::command]
pub async fn list_python_versions(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    project::list_python_versions(&config.tools.uv_path).await
}

/// Change la version Python du projet actif
#[tauri::command]
pub async fn set_project_python_version(
    state: State<'_, AppState>,
    version: String,
) -> Result<(), String> {
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    // Mettre à jour via uv
    project::set_python_version(&proj.name, &version, &config.tools.uv_path).await?;

    // Mettre à jour en DB
    db::update_project_python_version(&pool, project_id, &version).await?;

    Ok(())
}

// ============================================================================
// Project Benchmarks
// ============================================================================

/// Ajoute un benchmark au projet actif (chemin absolu)
#[tauri::command]
pub async fn add_benchmark_to_project(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Benchmark, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    // Valider le fichier
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("Fichier non trouvé: {file_path}"));
    }
    if path
        .extension()
        .is_none_or(|ext| !ext.eq_ignore_ascii_case("py"))
    {
        return Err("Le fichier doit être un fichier Python (.py)".to_string());
    }

    // Vérifier si déjà ajouté
    if db::benchmark_exists(&pool, project_id, &file_path).await? {
        return Err("Ce benchmark est déjà ajouté au projet".to_string());
    }

    // Extraire le nom
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Nom de fichier invalide")?
        .to_string();

    let id = db::insert_benchmark(&pool, project_id, &name, &file_path).await?;

    Ok(Benchmark {
        id,
        project_id,
        name,
        path: file_path,
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Supprime un benchmark du projet
#[tauri::command]
pub async fn remove_benchmark_from_project(
    state: State<'_, AppState>,
    benchmark_id: i64,
) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    db::delete_benchmark(&pool, benchmark_id).await
}

/// Liste les benchmarks du projet actif
#[tauri::command]
pub async fn list_project_benchmarks(state: State<'_, AppState>) -> Result<Vec<Benchmark>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    db::get_benchmarks_for_project(&pool, project_id).await
}

/// Analyse les dépendances Python d'un fichier benchmark
#[tauri::command]
pub async fn get_benchmark_dependencies(
    state: State<'_, AppState>,
    benchmark_path: String,
) -> Result<python_deps::DependencyAnalysis, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    let benchmark_path = Path::new(&benchmark_path);

    // Le dossier parent du benchmark comme racine pour les imports locaux
    let local_code_root = benchmark_path
        .parent()
        .ok_or("Impossible de déterminer le dossier parent")?;

    // pyproject.toml du projet
    let pyproject_path = project::pyproject_path(&proj.name)?;
    let pyproject = if pyproject_path.exists() {
        Some(pyproject_path.as_path())
    } else {
        None
    };

    let mut analyzer = python_deps::PythonAnalyzer::new()?;
    analyzer.analyze(benchmark_path, local_code_root, pyproject)
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif - sélectionnez un projet d'abord")?;

    let mut jobs = Vec::new();
    for name in benchmark_names {
        let id = db::insert_job(&pool, project_id, &name).await?;
        jobs.push(Job {
            id,
            project_id: Some(project_id),
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

    // Récupérer le projet actif
    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif - sélectionnez un projet d'abord")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    // Vérifier s'il y a déjà un job en cours
    if db::load_running_job(&pool).await?.is_some() {
        return Err("Un job est déjà en cours".to_string());
    }

    // Prendre le prochain job en attente
    let pending = db::load_pending_jobs(&pool).await?;
    if let Some(job) = pending.into_iter().next() {
        // Sync le projet avant de lancer (pyproject.toml, uv.lock)
        let project_dir = project::project_path(&proj.name)?;
        ssh::rsync_project_to_server(&config, &proj.name, &project_dir).await?;

        // Lancer le job
        ssh::start_tmux_job(&config, &proj.name, job.id, &job.benchmark_name).await?;

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

// ============================================================================
// Project Dependencies
// ============================================================================

/// Ajoute une dépendance au projet actif via `uv add`
#[tauri::command]
pub async fn add_project_dependency(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<String, String> {
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    project::add_dependency(&proj.name, &package_name, &config.tools.uv_path).await
}

/// Supprime une dépendance du projet actif via `uv remove`
#[tauri::command]
pub async fn remove_project_dependency(
    state: State<'_, AppState>,
    package_name: String,
) -> Result<String, String> {
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    project::remove_dependency(&proj.name, &package_name, &config.tools.uv_path).await
}

/// Met à jour toutes les dépendances du projet actif
#[tauri::command]
pub async fn update_project_dependencies(state: State<'_, AppState>) -> Result<String, String> {
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    project::update_dependencies(&proj.name, &config.tools.uv_path).await
}

/// Liste les dépendances du projet actif (depuis pyproject.toml)
#[tauri::command]
pub async fn list_project_dependencies(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("DB non initialisée")?
        .clone();

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    project::read_project_dependencies(&proj.name)
}

/// Synchronise l'environnement du projet actif via `uv sync`
#[tauri::command]
pub async fn sync_project_environment(state: State<'_, AppState>) -> Result<String, String> {
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

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("Aucun projet actif")?;

    let proj = db::get_project(&pool, project_id)
        .await?
        .ok_or("Projet non trouvé")?;

    project::sync_environment(&proj.name, &config.tools.uv_path).await
}
