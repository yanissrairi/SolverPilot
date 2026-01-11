use chrono::Utc;
use sqlx::{sqlite::SqlitePool, Row};

use crate::state::{Benchmark, Job, JobStatus, Project};

// =============================================================================
// Initialisation & Migrations
// =============================================================================

/// Initialise la base de données `SQLite` avec toutes les tables
pub async fn init_db(db_path: &str) -> Result<SqlitePool, String> {
    let pool = SqlitePool::connect(&format!("sqlite:{db_path}?mode=rwc"))
        .await
        .map_err(|e| format!("Erreur connexion SQLite: {e}"))?;

    // Table des projets
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            python_version TEXT NOT NULL DEFAULT '3.12',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        ",
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Erreur création table projects: {e}"))?;

    // Table des benchmarks (chemins absolus vers fichiers .py)
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS benchmarks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            UNIQUE(project_id, path)
        )
        ",
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Erreur création table benchmarks: {e}"))?;

    // Table des jobs
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER,
            benchmark_name TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('pending', 'running', 'completed', 'failed', 'killed')),
            created_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT,
            progress_current INTEGER DEFAULT 0,
            progress_total INTEGER DEFAULT 0,
            results_path TEXT,
            error_message TEXT,
            log_content TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id)
        )
        ",
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Erreur création table jobs: {e}"))?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .map_err(|e| format!("Erreur activation foreign keys: {e}"))?;

    // Run queue columns migration (Story 1.2 - Beta 1)
    migrate_queue_columns(&pool).await?;

    Ok(pool)
}

/// Migrates the jobs table to add queue-specific columns (Story 1.2)
/// This migration is idempotent - safe to run multiple times
pub async fn migrate_queue_columns(pool: &SqlitePool) -> Result<(), String> {
    // Check if columns already exist (idempotent migration)
    let has_queue_position = sqlx::query("SELECT queue_position FROM jobs LIMIT 1")
        .fetch_optional(pool)
        .await
        .is_ok();

    if !has_queue_position {
        // Add queue_position column (nullable - NULL for non-queued jobs)
        sqlx::query("ALTER TABLE jobs ADD COLUMN queue_position INTEGER")
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to add queue_position column: {e}"))?;

        // Add queued_at column (nullable - NULL for non-queued jobs)
        sqlx::query("ALTER TABLE jobs ADD COLUMN queued_at TEXT")
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to add queued_at column: {e}"))?;
    }

    Ok(())
}

// =============================================================================
// Projects CRUD
// =============================================================================

/// Insère un nouveau projet
pub async fn insert_project(
    pool: &SqlitePool,
    name: &str,
    python_version: &str,
) -> Result<i64, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r"
        INSERT INTO projects (name, python_version, created_at, updated_at)
        VALUES (?, ?, ?, ?)
        ",
    )
    .bind(name)
    .bind(python_version)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Erreur insertion projet: {e}"))?;

    Ok(result.last_insert_rowid())
}

/// Récupère un projet par ID
pub async fn get_project(pool: &SqlitePool, id: i64) -> Result<Option<Project>, String> {
    let row = sqlx::query(
        r"
        SELECT id, name, python_version, created_at, updated_at
        FROM projects WHERE id = ?
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Erreur chargement projet: {e}"))?;

    Ok(row.map(|r| Project {
        id: r.get("id"),
        name: r.get("name"),
        python_version: r.get("python_version"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

/// Liste tous les projets
pub async fn list_projects(pool: &SqlitePool) -> Result<Vec<Project>, String> {
    let rows = sqlx::query(
        r"
        SELECT id, name, python_version, created_at, updated_at
        FROM projects ORDER BY name ASC
        ",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Erreur chargement projets: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|r| Project {
            id: r.get("id"),
            name: r.get("name"),
            python_version: r.get("python_version"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect())
}

/// Met à jour la version Python d'un projet
pub async fn update_project_python_version(
    pool: &SqlitePool,
    id: i64,
    version: &str,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();

    sqlx::query("UPDATE projects SET python_version = ?, updated_at = ? WHERE id = ?")
        .bind(version)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur mise à jour version Python: {e}"))?;

    Ok(())
}

/// Supprime un projet (cascade sur benchmarks)
pub async fn delete_project(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM projects WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur suppression projet: {e}"))?;

    Ok(())
}

// =============================================================================
// Benchmarks CRUD
// =============================================================================

/// Insère un nouveau benchmark
pub async fn insert_benchmark(
    pool: &SqlitePool,
    project_id: i64,
    name: &str,
    path: &str,
) -> Result<i64, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r"
        INSERT INTO benchmarks (project_id, name, path, created_at)
        VALUES (?, ?, ?, ?)
        ",
    )
    .bind(project_id)
    .bind(name)
    .bind(path)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Erreur insertion benchmark: {e}"))?;

    Ok(result.last_insert_rowid())
}

/// Liste les benchmarks d'un projet
pub async fn get_benchmarks_for_project(
    pool: &SqlitePool,
    project_id: i64,
) -> Result<Vec<Benchmark>, String> {
    let rows = sqlx::query(
        r"
        SELECT id, project_id, name, path, created_at
        FROM benchmarks WHERE project_id = ?
        ORDER BY name ASC
        ",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Erreur chargement benchmarks: {e}"))?;

    Ok(rows
        .into_iter()
        .map(|r| Benchmark {
            id: r.get("id"),
            project_id: r.get("project_id"),
            name: r.get("name"),
            path: r.get("path"),
            created_at: r.get("created_at"),
        })
        .collect())
}

/// Vérifie si un benchmark existe déjà (par path)
pub async fn benchmark_exists(
    pool: &SqlitePool,
    project_id: i64,
    path: &str,
) -> Result<bool, String> {
    let row = sqlx::query("SELECT 1 FROM benchmarks WHERE project_id = ? AND path = ?")
        .bind(project_id)
        .bind(path)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Erreur vérification benchmark: {e}"))?;

    Ok(row.is_some())
}

/// Supprime un benchmark
pub async fn delete_benchmark(pool: &SqlitePool, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM benchmarks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur suppression benchmark: {e}"))?;

    Ok(())
}

// =============================================================================
// Queue Helper Functions (Story 1.2 - Beta 1)
// =============================================================================

/// Gets the maximum queue position currently assigned
/// Returns 0 if no jobs are queued
pub async fn get_max_queue_position(pool: &SqlitePool) -> Result<i64, String> {
    let row = sqlx::query("SELECT COALESCE(MAX(queue_position), 0) as max_pos FROM jobs")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to get max queue position: {e}"))?;

    Ok(row.get("max_pos"))
}

/// Inserts a new job with queue position and `queued_at` timestamp
pub async fn insert_job_with_queue(
    pool: &SqlitePool,
    project_id: i64,
    benchmark_name: &str,
    queue_position: i64,
    queued_at: &str,
) -> Result<i64, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r"
        INSERT INTO jobs (project_id, benchmark_name, status, created_at, queue_position, queued_at)
        VALUES (?, ?, 'pending', ?, ?, ?)
        ",
    )
    .bind(project_id)
    .bind(benchmark_name)
    .bind(&now)
    .bind(queue_position)
    .bind(queued_at)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to insert job with queue: {e}"))?;

    Ok(result.last_insert_rowid())
}

/// Gets a benchmark by ID
pub async fn get_benchmark_by_id(pool: &SqlitePool, id: i64) -> Result<Benchmark, String> {
    let row = sqlx::query(
        r"
        SELECT id, project_id, name, path, created_at
        FROM benchmarks WHERE id = ?
        ",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to get benchmark by ID: {e}"))?;

    Ok(Benchmark {
        id: row.get("id"),
        project_id: row.get("project_id"),
        name: row.get("name"),
        path: row.get("path"),
        created_at: row.get("created_at"),
    })
}

/// Gets all queued jobs ordered by `queue_position`
pub async fn get_queued_jobs(pool: &SqlitePool) -> Result<Vec<Job>, String> {
    let rows = sqlx::query(
        r"
        SELECT id, project_id, benchmark_name, status, created_at, started_at, finished_at,
               progress_current, progress_total, results_path, error_message, log_content,
               queue_position, queued_at
        FROM jobs
        WHERE queue_position IS NOT NULL
        ORDER BY queue_position ASC
        ",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get queued jobs: {e}"))?;

    Ok(rows_to_jobs_with_queue(rows))
}

// =============================================================================
// Jobs (existant)
// =============================================================================

const fn status_to_str(status: &JobStatus) -> &'static str {
    match status {
        JobStatus::Pending => "pending",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Killed => "killed",
    }
}

fn str_to_status(s: &str) -> JobStatus {
    match s {
        "running" => JobStatus::Running,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        "killed" => JobStatus::Killed,
        _ => JobStatus::Pending, // "pending" ou valeur inconnue → Pending par défaut
    }
}

/// Insère un nouveau job
pub async fn insert_job(
    pool: &SqlitePool,
    project_id: i64,
    benchmark_name: &str,
) -> Result<i64, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r"
        INSERT INTO jobs (project_id, benchmark_name, status, created_at)
        VALUES (?, ?, 'pending', ?)
        ",
    )
    .bind(project_id)
    .bind(benchmark_name)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| format!("Erreur insertion job: {e}"))?;

    Ok(result.last_insert_rowid())
}

/// Met à jour le statut d'un job
pub async fn update_job_status(
    pool: &SqlitePool,
    job_id: i64,
    status: &JobStatus,
) -> Result<(), String> {
    let status_str = status_to_str(status);
    let now = Utc::now().to_rfc3339();

    let query = match status {
        JobStatus::Running => {
            sqlx::query("UPDATE jobs SET status = ?, started_at = ? WHERE id = ?")
                .bind(status_str)
                .bind(&now)
                .bind(job_id)
        }
        JobStatus::Completed | JobStatus::Failed | JobStatus::Killed => {
            sqlx::query("UPDATE jobs SET status = ?, finished_at = ? WHERE id = ?")
                .bind(status_str)
                .bind(&now)
                .bind(job_id)
        }
        JobStatus::Pending => sqlx::query("UPDATE jobs SET status = ? WHERE id = ?")
            .bind(status_str)
            .bind(job_id),
    };

    query
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur mise à jour statut: {e}"))?;

    Ok(())
}

/// Met à jour la progression d'un job
pub async fn update_job_progress(
    pool: &SqlitePool,
    job_id: i64,
    current: u32,
    total: u32,
) -> Result<(), String> {
    #[allow(clippy::cast_possible_wrap)]
    let current_i32 = current as i32;
    #[allow(clippy::cast_possible_wrap)]
    let total_i32 = total as i32;

    sqlx::query("UPDATE jobs SET progress_current = ?, progress_total = ? WHERE id = ?")
        .bind(current_i32)
        .bind(total_i32)
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur mise à jour progression: {e}"))?;

    Ok(())
}

/// Met à jour les logs d'un job
pub async fn update_job_logs(pool: &SqlitePool, job_id: i64, logs: &str) -> Result<(), String> {
    sqlx::query("UPDATE jobs SET log_content = ? WHERE id = ?")
        .bind(logs)
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur mise à jour logs: {e}"))?;

    Ok(())
}

/// Met à jour l'erreur d'un job
pub async fn update_job_error(pool: &SqlitePool, job_id: i64, error: &str) -> Result<(), String> {
    sqlx::query("UPDATE jobs SET error_message = ? WHERE id = ?")
        .bind(error)
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur mise à jour erreur: {e}"))?;

    Ok(())
}

/// Supprime un job en attente
pub async fn delete_pending_job(pool: &SqlitePool, job_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM jobs WHERE id = ? AND status = 'pending'")
        .bind(job_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Erreur suppression job: {e}"))?;

    Ok(())
}

/// Charge les jobs en attente
pub async fn load_pending_jobs(pool: &SqlitePool) -> Result<Vec<Job>, String> {
    load_jobs_by_status(pool, "pending").await
}

/// Charge le job en cours
pub async fn load_running_job(pool: &SqlitePool) -> Result<Option<Job>, String> {
    let jobs = load_jobs_by_status(pool, "running").await?;
    Ok(jobs.into_iter().next())
}

/// Charge l'historique (jobs terminés)
pub async fn load_history(pool: &SqlitePool, limit: i32) -> Result<Vec<Job>, String> {
    let rows = sqlx::query(
        r"
        SELECT id, project_id, benchmark_name, status, created_at, started_at, finished_at,
               progress_current, progress_total, results_path, error_message, log_content
        FROM jobs
        WHERE status IN ('completed', 'failed', 'killed')
        ORDER BY finished_at DESC
        LIMIT ?
        ",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Erreur chargement historique: {e}"))?;

    Ok(rows_to_jobs(rows))
}

async fn load_jobs_by_status(pool: &SqlitePool, status: &str) -> Result<Vec<Job>, String> {
    let rows = sqlx::query(
        r"
        SELECT id, project_id, benchmark_name, status, created_at, started_at, finished_at,
               progress_current, progress_total, results_path, error_message, log_content
        FROM jobs
        WHERE status = ?
        ORDER BY created_at ASC
        ",
    )
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Erreur chargement jobs: {e}"))?;

    Ok(rows_to_jobs(rows))
}

fn rows_to_jobs(rows: Vec<sqlx::sqlite::SqliteRow>) -> Vec<Job> {
    let mut jobs = Vec::new();

    for row in rows {
        let id: i64 = row.get("id");
        let project_id: Option<i64> = row.get("project_id");
        let benchmark_name: String = row.get("benchmark_name");
        let status_str: String = row.get("status");
        let created_at: String = row.get("created_at");
        let started_at: Option<String> = row.get("started_at");
        let finished_at: Option<String> = row.get("finished_at");
        let progress_current: i32 = row.get("progress_current");
        let progress_total: i32 = row.get("progress_total");
        let results_path: Option<String> = row.get("results_path");
        let error_message: Option<String> = row.get("error_message");
        let log_content: Option<String> = row.get("log_content");

        #[allow(clippy::cast_sign_loss)]
        let progress_current_u32 = progress_current as u32;
        #[allow(clippy::cast_sign_loss)]
        let progress_total_u32 = progress_total as u32;

        jobs.push(Job {
            id,
            project_id,
            benchmark_name,
            status: str_to_status(&status_str),
            created_at,
            started_at,
            finished_at,
            progress_current: progress_current_u32,
            progress_total: progress_total_u32,
            results_path,
            error_message,
            log_content: log_content.unwrap_or_default(),
            queue_position: None,
            queued_at: None,
        });
    }

    jobs
}

/// Converts database rows to Job structs, including queue fields (Story 1.2)
fn rows_to_jobs_with_queue(rows: Vec<sqlx::sqlite::SqliteRow>) -> Vec<Job> {
    let mut jobs = Vec::new();

    for row in rows {
        let id: i64 = row.get("id");
        let project_id: Option<i64> = row.get("project_id");
        let benchmark_name: String = row.get("benchmark_name");
        let status_str: String = row.get("status");
        let created_at: String = row.get("created_at");
        let started_at: Option<String> = row.get("started_at");
        let finished_at: Option<String> = row.get("finished_at");
        let progress_current: i32 = row.get("progress_current");
        let progress_total: i32 = row.get("progress_total");
        let results_path: Option<String> = row.get("results_path");
        let error_message: Option<String> = row.get("error_message");
        let log_content: Option<String> = row.get("log_content");
        let queue_position: Option<i64> = row.get("queue_position");
        let queued_at: Option<String> = row.get("queued_at");

        #[allow(clippy::cast_sign_loss)]
        let progress_current_u32 = progress_current as u32;
        #[allow(clippy::cast_sign_loss)]
        let progress_total_u32 = progress_total as u32;

        jobs.push(Job {
            id,
            project_id,
            benchmark_name,
            status: str_to_status(&status_str),
            created_at,
            started_at,
            finished_at,
            progress_current: progress_current_u32,
            progress_total: progress_total_u32,
            results_path,
            error_message,
            log_content: log_content.unwrap_or_default(),
            queue_position,
            queued_at,
        });
    }

    jobs
}
