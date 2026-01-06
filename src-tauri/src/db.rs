use chrono::Utc;
use sqlx::{sqlite::SqlitePool, Row};

use crate::state::{Job, JobStatus};

/// Initialise la base de données `SQLite`
pub async fn init_db(db_path: &str) -> Result<SqlitePool, String> {
    let pool = SqlitePool::connect(&format!("sqlite:{db_path}?mode=rwc"))
        .await
        .map_err(|e| format!("Erreur connexion SQLite: {e}"))?;

    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS jobs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            benchmark_name TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('pending', 'running', 'completed', 'failed', 'killed')),
            created_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT,
            progress_current INTEGER DEFAULT 0,
            progress_total INTEGER DEFAULT 0,
            results_path TEXT,
            error_message TEXT,
            log_content TEXT
        )
        ",
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Erreur création table jobs: {e}"))?;

    Ok(pool)
}

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
pub async fn insert_job(pool: &SqlitePool, benchmark_name: &str) -> Result<i64, String> {
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r"
        INSERT INTO jobs (benchmark_name, status, created_at)
        VALUES (?, 'pending', ?)
        ",
    )
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
        SELECT id, benchmark_name, status, created_at, started_at, finished_at,
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
        SELECT id, benchmark_name, status, created_at, started_at, finished_at,
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
        });
    }

    jobs
}
