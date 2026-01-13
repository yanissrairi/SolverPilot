use std::path::Path;
use std::sync::Arc;
use tauri::State;

use crate::config::AppConfig;
use crate::ssh::SshKeyStatus;
use crate::state::{AppState, Benchmark, Job, JobStatus, JobStatusResponse, Project, SyncStatus};
use crate::{db, job, project, python_deps, ssh};

// Helper macro to get SSH manager from state
macro_rules! get_ssh_manager {
    ($state:expr) => {
        $state
            .ssh_manager
            .lock()
            .await
            .as_ref()
            .ok_or("SSH not initialized - call init_ssh first")?
            .clone()
    };
}

// ============================================================================
// Configuration
// ============================================================================

/// Vérifie si le fichier de configuration existe
#[tauri::command]
pub fn check_config_exists() -> Result<bool, String> {
    AppConfig::exists()
}

/// Retourne le chemin du fichier de configuration
#[tauri::command]
pub fn get_config_path() -> Result<String, String> {
    Ok(crate::paths::config_path()?.display().to_string())
}

/// Sauvegarde la configuration et la charge dans l'état
#[tauri::command]
pub async fn save_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    config.save()?;

    // Initialiser la DB si pas déjà fait
    let db_path = AppConfig::db_path()?;
    let pool = db::init_db(&db_path.to_string_lossy()).await?;

    // Stocker dans l'état
    *state.config.lock().await = Some(config);
    *state.db.lock().await = Some(pool);

    Ok(())
}

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

    // Create SSH authentication
    let key_path = ssh::get_ssh_key_path(&config);
    let auth = ssh::SshAuth::key(key_path);

    // Create SSH manager with connection pool (size: 10)
    let manager = ssh::SshManager::new(config.clone(), auth, 10)
        .await
        .map_err(|e| e.to_string())?;

    let info = format!(
        "SSH manager initialized for {}:{}",
        config.ssh.host, config.ssh.port
    );

    // Store manager in state
    *state.ssh_manager.lock().await = Some(manager);

    Ok(info)
}

#[tauri::command]
pub async fn close_ssh(state: State<'_, AppState>) -> Result<(), String> {
    // Drop the SSH manager (closes all connections)
    *state.ssh_manager.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn test_ssh(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = state
        .ssh_manager
        .lock()
        .await
        .as_ref()
        .ok_or("SSH not initialized")?
        .clone();

    manager.test_connection().await.map_err(|e| e.to_string())?;
    Ok(true)
}

/// Test SSH direct (sans pool) - pour le wizard de setup
#[tauri::command]
pub async fn test_ssh_direct(
    state: State<'_, AppState>,
    passphrase: Option<String>,
) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let key_path = ssh::get_ssh_key_path(&config);

    // Use passphrase if provided, otherwise try without
    let auth = if let Some(pass) = passphrase {
        ssh::SshAuth::key_with_passphrase(key_path, pass)
    } else {
        ssh::SshAuth::key(key_path)
    };

    ssh::test_connection_direct(&config, &auth)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_ssh_key_status(state: State<'_, AppState>) -> Result<SshKeyStatus, String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let key_path = ssh::get_ssh_key_path(&config);
    let auth = ssh::SshAuth::key(key_path);

    ssh::check_key_status(&config, &auth).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_ssh_key(state: State<'_, AppState>, passphrase: String) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    // Re-initialize SSH manager with passphrase
    let key_path = ssh::get_ssh_key_path(&config);
    let auth = ssh::SshAuth::key_with_passphrase(key_path, passphrase);

    let manager = ssh::SshManager::new(config.clone(), auth, 10)
        .await
        .map_err(|e| e.to_string())?;

    *state.ssh_manager.lock().await = Some(manager);

    Ok(())
}

// ============================================================================
// Server Database
// ============================================================================

/// Initialize server-side database via SSH
/// Generates SQL script and executes on remote server at ~/.solverpilot-server/server.db
#[tauri::command]
pub async fn init_server_db(state: State<'_, AppState>) -> Result<(), String> {
    let manager = get_ssh_manager!(state);

    // Generate SQL initialization script
    let init_script = crate::server_db::generate_init_script();

    // Create remote directory
    let mkdir_cmd = "mkdir -p ~/.solverpilot-server";
    manager
        .executor()
        .execute(mkdir_cmd)
        .await
        .map_err(|e| format!("Failed to create server directory: {e}"))?;

    // Execute SQL script on remote server
    // Using heredoc to avoid escaping issues
    let sql_cmd =
        format!("sqlite3 ~/.solverpilot-server/server.db <<'SQL_EOF'\n{init_script}\nSQL_EOF");

    manager
        .executor()
        .execute(&sql_cmd)
        .await
        .map_err(|e| format!("Failed to initialize server database: {e}"))?;

    // Set database permissions to 0600
    let chmod_cmd = "chmod 600 ~/.solverpilot-server/server.db";
    manager
        .executor()
        .execute(chmod_cmd)
        .await
        .map_err(|e| format!("Failed to set database permissions: {e}"))?;

    tracing::info!("Server database initialized at ~/.solverpilot-server/server.db");

    Ok(())
}

// ============================================================================
// Wrapper Deployment
// ============================================================================

/// Check if wrapper script is installed on remote server
#[tauri::command]
pub async fn check_wrapper_installed(state: State<'_, AppState>) -> Result<bool, String> {
    let manager = get_ssh_manager!(state);
    let wrapper_mgr = crate::wrapper::WrapperManager::new();

    wrapper_mgr.check_installed(manager.executor()).await
}

/// Deploy wrapper script to remote server
/// Also initializes server database if not already done
#[tauri::command]
pub async fn deploy_wrapper(state: State<'_, AppState>) -> Result<(), String> {
    let manager = get_ssh_manager!(state);
    let wrapper_mgr = crate::wrapper::WrapperManager::new();

    // Check if already installed (idempotent)
    if wrapper_mgr.check_installed(manager.executor()).await? {
        tracing::debug!("Wrapper already installed, skipping deployment");
        return Ok(());
    }

    // Deploy wrapper script
    wrapper_mgr.deploy_to_server(manager.executor()).await?;

    // Initialize server database
    init_server_db(state).await?;

    // Update wrapper version in metadata (using heredoc to avoid SQL injection risks)
    let update_version_cmd = format!(
        "sqlite3 ~/.solverpilot-server/server.db <<'SQL_EOF'\nINSERT OR REPLACE INTO metadata (key, value) VALUES ('wrapper_version', '{}');\nSQL_EOF",
        crate::wrapper::WRAPPER_VERSION
    );

    manager
        .executor()
        .execute(&update_version_cmd)
        .await
        .map_err(|e| format!("Failed to update wrapper version in metadata: {e}"))?;

    tracing::info!(
        "Wrapper deployment complete (version {})",
        crate::wrapper::WRAPPER_VERSION
    );

    Ok(())
}

// ============================================================================
// Sync
// ============================================================================

#[tauri::command]
pub async fn check_sync_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    let _config = state
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

    match get_ssh_manager!(state)
        .transfer()
        .dry_run_project(&proj.name, &project_dir)
        .await
    {
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
        Err(e) => Ok(SyncStatus::Error {
            message: e.to_string(),
        }),
    }
}

#[tauri::command]
pub async fn sync_code(state: State<'_, AppState>) -> Result<(), String> {
    let _config = state
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

    get_ssh_manager!(state)
        .transfer()
        .rsync_project(&proj.name, &project_dir)
        .await
        .map_err(|e| e.to_string())
}

/// Synchronise uniquement les fichiers nécessaires pour un benchmark
/// (analyse les dépendances et sync les fichiers identifiés)
#[tauri::command]
pub async fn sync_benchmark_deps(
    state: State<'_, AppState>,
    benchmark_path: String,
) -> Result<usize, String> {
    let _config = state
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
    get_ssh_manager!(state)
        .transfer()
        .rsync_project(&proj.name, &project_dir)
        .await
        .map_err(|e| e.to_string())?;

    // Puis sync les fichiers du benchmark
    get_ssh_manager!(state)
        .transfer()
        .rsync_benchmarks(&proj.name, benchmark_path, files)
        .await
        .map_err(|e| e.to_string())?;

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
            queue_position: None, // Alpha behavior - no queue
            queued_at: None,
        });
    }

    Ok(jobs)
}

/// Queue benchmarks by their IDs with queue position and timestamp (Story 1.2)
/// Enhanced with duplicate detection in Story 1.5
/// Uses a transaction to ensure atomicity (NFR-R7) - all jobs are queued or none are.
#[tauri::command]
pub async fn queue_benchmarks(
    state: State<'_, AppState>,
    benchmark_ids: Vec<i64>,
    force_duplicate: Option<bool>,
) -> Result<Vec<Job>, String> {
    // Default force_duplicate to false if not provided
    let force_duplicate = force_duplicate.unwrap_or(false);
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    let config = state
        .config
        .lock()
        .await
        .as_ref()
        .ok_or("Config not loaded")?
        .clone();

    let project_id = state
        .current_project_id
        .lock()
        .await
        .ok_or("No active project - select a project first")?;

    // Duplicate detection check (Story 1.5)
    // Only check if force_duplicate is false
    if !force_duplicate {
        for bench_id in &benchmark_ids {
            let benchmark = db::get_benchmark_by_id(&pool, *bench_id).await?;
            let dup_check = db::check_duplicate_job(&pool, &benchmark.name).await?;

            if dup_check.is_duplicate {
                use crate::config::DuplicateHandling;

                match config.queue_settings.duplicate_handling {
                    DuplicateHandling::Prevent => {
                        return Err(format!(
                            "{} is already queued. Duplicates are not allowed.",
                            benchmark.name
                        ));
                    }
                    DuplicateHandling::Allow => {
                        // Continue to queue without warning
                    }
                    DuplicateHandling::Warn => {
                        // Return special error that frontend handles with confirmation dialog
                        return Err(format!(
                            "DUPLICATE_WARNING:{}:{}",
                            benchmark.name,
                            dup_check.existing_statuses.join(",")
                        ));
                    }
                }
            }
        }
    }

    // Begin transaction for atomic batch insertion (NFR-R7)
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("Failed to begin transaction: {e}"))?;

    // Get current max queue position (within transaction for consistency)
    let max_pos: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(queue_position), 0) FROM jobs")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to get max queue position: {e}"))?;

    let mut jobs = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for (idx, bench_id) in benchmark_ids.iter().enumerate() {
        let benchmark = db::get_benchmark_by_id(&pool, *bench_id).await?;

        // Safe: benchmark queue size will never exceed i64::MAX in practice
        // Typical queue sizes are <1000 jobs, well within i64 range
        #[allow(clippy::cast_possible_wrap)]
        let queue_pos = max_pos + (idx as i64) + 1;

        // Insert job within transaction
        let job_id: i64 = sqlx::query_scalar(
            r"
            INSERT INTO jobs (project_id, benchmark_name, status, created_at, queue_position, queued_at)
            VALUES (?, ?, 'pending', ?, ?, ?)
            RETURNING id
            ",
        )
        .bind(project_id)
        .bind(&benchmark.name)
        .bind(&now)
        .bind(queue_pos)
        .bind(&now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Failed to insert job: {e}"))?;

        jobs.push(Job {
            id: job_id,
            project_id: Some(project_id),
            benchmark_name: benchmark.name,
            status: JobStatus::Pending,
            created_at: now.clone(),
            started_at: None,
            finished_at: None,
            progress_current: 0,
            progress_total: 0,
            results_path: None,
            error_message: None,
            log_content: String::new(),
            queue_position: Some(queue_pos),
            queued_at: Some(now.clone()),
        });
    }

    // Commit transaction - all jobs queued atomically
    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit transaction: {e}"))?;

    Ok(jobs)
}

/// Get all queued jobs ordered by status priority (Story 1.3)
/// Returns jobs sorted: running → pending → completed/failed → killed
#[tauri::command]
pub async fn get_all_queue_jobs(state: State<'_, AppState>) -> Result<Vec<Job>, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::get_queued_jobs(&pool).await
}

/// Remove a job from the queue (Story 1.4)
/// Only pending jobs can be removed. Running/completed jobs are protected.
#[tauri::command]
pub async fn remove_job_from_queue(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::remove_job_from_queue(&pool, job_id).await
}

/// Move a job to the front of the queue (Story 1.4)
/// Only pending jobs can be moved. Sets `queue_position` to 1.
#[tauri::command]
pub async fn move_job_to_front(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::move_job_to_front(&pool, job_id).await
}

/// Move a job to the end of the queue (Story 1.4)
/// Only pending jobs can be moved. Sets `queue_position` to max+1.
#[tauri::command]
pub async fn move_job_to_end(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::move_job_to_end(&pool, job_id).await
}

/// Reorder a job to a new position in the queue (Story 1.4)
/// Only pending jobs can be reordered. Shifts other jobs accordingly.
#[tauri::command]
pub async fn reorder_queue_job(
    state: State<'_, AppState>,
    job_id: i64,
    new_position: i32,
) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::reorder_queue_job(&pool, job_id, new_position).await
}

/// Cancel all pending jobs in the queue (Story 1.4)
/// Running and completed jobs are preserved. Returns count of deleted jobs.
#[tauri::command]
pub async fn cancel_all_pending_jobs(state: State<'_, AppState>) -> Result<u32, String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    db::cancel_all_pending_jobs(&pool).await
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
        // Story 2.3: Auto-deploy wrapper on first queue execution
        // Check if wrapper is installed, deploy if not (idempotent)
        let manager = get_ssh_manager!(state);
        let wrapper_mgr = crate::wrapper::WrapperManager::new();

        if !wrapper_mgr.check_installed(manager.executor()).await? {
            tracing::info!("Wrapper not installed, deploying infrastructure...");
            // Deploy wrapper script
            wrapper_mgr.deploy_to_server(manager.executor()).await?;

            // Initialize server database
            let init_script = crate::server_db::generate_init_script();
            manager
                .executor()
                .execute("mkdir -p ~/.solverpilot-server")
                .await
                .map_err(|e| format!("Failed to create server directory: {e}"))?;

            let sql_cmd = format!(
                "sqlite3 ~/.solverpilot-server/server.db <<'SQL_EOF'\n{init_script}\nSQL_EOF"
            );
            manager
                .executor()
                .execute(&sql_cmd)
                .await
                .map_err(|e| format!("Failed to initialize server database: {e}"))?;

            manager
                .executor()
                .execute("chmod 600 ~/.solverpilot-server/server.db")
                .await
                .map_err(|e| format!("Failed to set database permissions: {e}"))?;

            // Update wrapper version in metadata
            let version_cmd = format!(
                "sqlite3 ~/.solverpilot-server/server.db <<'SQL_EOF'\nINSERT OR REPLACE INTO metadata (key, value) VALUES ('wrapper_version', '{}');\nSQL_EOF",
                crate::wrapper::WRAPPER_VERSION
            );
            manager
                .executor()
                .execute(&version_cmd)
                .await
                .map_err(|e| format!("Failed to update wrapper version: {e}"))?;

            tracing::info!("Queue infrastructure deployed successfully");
        }

        // Sync le projet avant de lancer (pyproject.toml, uv.lock)
        let project_dir = project::project_path(&proj.name)?;
        get_ssh_manager!(state)
            .transfer()
            .rsync_project(&proj.name, &project_dir)
            .await
            .map_err(|e| e.to_string())?;

        // Lancer le job via tmux
        let jobs_path = config.remote_jobs_path();
        let log_file = format!("{}/{}.log", jobs_path, job.id);
        let project_dir = format!("{}/projects/{}", config.remote.remote_base, proj.name);
        let uv_path = &config.tools.uv_path;

        let gurobi_exports = if config.gurobi.home.is_empty() {
            String::new()
        } else {
            format!(
                r#"export GUROBI_HOME="{}"; export GRB_LICENSE_FILE="{}"; export PATH="$PATH:$GUROBI_HOME/bin"; export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$GUROBI_HOME/lib"; "#,
                config.gurobi.home, config.gurobi.license_file
            )
        };

        let cmd = format!(
            r#"tmux new-session -d -s job_{} 'exec > {} 2>&1; export PYTHONUNBUFFERED=1; {}cd {} && echo "=== Starting job ===" && echo "Working directory: $(pwd)" && echo "=== uv sync ===" && {} sync && echo "=== Running benchmark ===" && {} run python code/{} ; echo "=== Job finished with code: $? ==="'"#,
            job.id, log_file, gurobi_exports, project_dir, uv_path, uv_path, job.benchmark_name
        );

        get_ssh_manager!(state)
            .executor()
            .execute_background(&cmd)
            .await
            .map_err(|e| e.to_string())?;

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
    let _config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config non chargée")?;

    let job_id = *state.current_job_id.lock().await;
    if let Some(job_id) = job_id {
        get_ssh_manager!(state)
            .executor()
            .tmux_send_ctrl_c(&format!("job_{job_id}"))
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn kill_job(state: State<'_, AppState>) -> Result<(), String> {
    let _config = state
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
        get_ssh_manager!(state)
            .executor()
            .tmux_kill_session(&format!("job_{job_id}"))
            .await
            .map_err(|e| e.to_string())?;
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
        get_ssh_manager!(state)
            .executor()
            .tail_logs(
                &format!("{}/jobs/{}.log", config.remote.remote_base, job_id),
                lines,
            )
            .await
            .map_err(|e| e.to_string())
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
        let logs = get_ssh_manager!(state)
            .executor()
            .tail_logs(
                &format!("{}/jobs/{}.log", config.remote.remote_base, job_id),
                200,
            )
            .await
            .map_err(|e| e.to_string())
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
        let tmux_exists = get_ssh_manager!(state)
            .executor()
            .tmux_session_exists(&session_name)
            .await
            .map_err(|e| e.to_string())
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

// ============================================================================
// Queue Processing (Story 2.4)
// ============================================================================

/// Start automated queue processing (sequential execution)
///
/// Spawns a background task that processes jobs one at a time (FIFO order).
/// Jobs are automatically started after previous job completes.
/// Queue stops when empty or `stop_queue_processing()` is called.
#[tauri::command]
pub async fn start_queue_processing(state: State<'_, AppState>) -> Result<(), String> {
    let config = state
        .config
        .lock()
        .await
        .clone()
        .ok_or("Config not loaded")?;

    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    let ssh_manager = state
        .ssh_manager
        .lock()
        .await
        .as_ref()
        .ok_or("SSH not connected")?
        .clone();

    let queue_manager = state.queue_manager.lock().await.clone();

    // Check if already processing
    if queue_manager.is_processing().await {
        return Err("Queue is already processing".to_string());
    }

    // Start processing with config values
    queue_manager
        .start_processing(
            pool,
            Arc::new(ssh_manager),
            config.ssh.host,
            config.ssh.user,
        )
        .await?;

    tracing::info!("Queue processing started");
    Ok(())
}

/// Stop queue processing gracefully
///
/// Stops processing after current job completes.
/// Does not cancel the running job.
#[tauri::command]
pub async fn stop_queue_processing(state: State<'_, AppState>) -> Result<(), String> {
    let queue_manager = state.queue_manager.lock().await.clone();
    queue_manager.stop_processing().await?;
    tracing::info!("Queue processing will stop after current job");
    Ok(())
}

/// Pause queue processing (graceful)
///
/// Running jobs complete naturally, new jobs don't start.
/// Can only pause if currently running.
#[tauri::command]
pub async fn pause_queue_processing(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    let queue_manager = state.queue_manager.lock().await.clone();
    queue_manager.pause_processing(&pool).await?;

    tracing::info!("Queue processing paused");
    Ok(())
}

/// Resume queue processing from paused state
///
/// Can only resume if currently paused.
#[tauri::command]
pub async fn resume_queue_processing(state: State<'_, AppState>) -> Result<(), String> {
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    let queue_manager = state.queue_manager.lock().await.clone();
    queue_manager.resume_processing(&pool).await?;

    tracing::info!("Queue processing resumed");
    Ok(())
}

/// Get current queue processing status
///
/// Returns:
/// - `state`: Queue state (idle/running/paused)
/// - `currentJobId`: ID of currently executing job (if any)
/// - `pendingCount`: Number of pending jobs in queue
/// - `runningCount`: Number of running jobs
/// - `completedCount`: Number of completed jobs
#[tauri::command]
pub async fn get_queue_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let queue_manager = state.queue_manager.lock().await.clone();

    let queue_state = queue_manager.get_state().await;
    let current_job_id = queue_manager.current_job().await;

    // Get job counts from database
    let pool = state
        .db
        .lock()
        .await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    let pending_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status = 'pending'")
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to count pending jobs: {e}"))?;

    let running_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status = 'running'")
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to count running jobs: {e}"))?;

    let completed_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status = 'completed'")
            .fetch_one(&pool)
            .await
            .map_err(|e| format!("Failed to count completed jobs: {e}"))?;

    Ok(serde_json::json!({
        "state": queue_state.as_str(),
        "currentJobId": current_job_id,
        "pendingCount": pending_count,
        "runningCount": running_count,
        "completedCount": completed_count,
    }))
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
