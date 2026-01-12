#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::os::unix::fs::PermissionsExt;

const SCHEMA: &str = include_str!("../sql/server_schema.sql");

/// Generate the SQL initialization script for the server database
/// This script includes PRAGMAs, table creation, indexes, and metadata
pub fn generate_init_script() -> String {
    SCHEMA.to_string()
}

/// Initialize server database for local testing only
/// Production deployment uses SSH execution (Story 2.3)
#[cfg(test)]
pub async fn init_local_test_db(db_path: &str) -> Result<(), String> {
    use sqlx::sqlite::SqlitePool;

    // Create database file with restricted permissions
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Connect and execute schema
    let pool = SqlitePool::connect(&format!("sqlite:{db_path}?mode=rwc"))
        .await
        .map_err(|e| format!("Failed to connect to SQLite: {e}"))?;

    // Execute schema (includes PRAGMAs, CREATE TABLE, indexes)
    sqlx::query(SCHEMA)
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to execute schema: {e}"))?;

    pool.close().await;

    // Set file permissions to 0600 (user read/write only)
    #[cfg(unix)]
    {
        let file =
            fs::File::open(db_path).map_err(|e| format!("Failed to open database file: {e}"))?;
        let mut perms = file
            .metadata()
            .map_err(|e| format!("Failed to get metadata: {e}"))?
            .permissions();
        perms.set_mode(0o600);
        fs::set_permissions(db_path, perms)
            .map_err(|e| format!("Failed to set permissions: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_init_script() {
        let script = generate_init_script();

        // Verify script contains essential components
        assert!(script.contains("PRAGMA journal_mode = WAL"));
        assert!(script.contains("PRAGMA busy_timeout = 5000"));
        assert!(script.contains("PRAGMA foreign_keys = ON"));
        assert!(script.contains("CREATE TABLE IF NOT EXISTS jobs"));
        assert!(script.contains("CREATE INDEX IF NOT EXISTS idx_jobs_status"));
        assert!(script.contains("CREATE INDEX IF NOT EXISTS idx_jobs_user"));
        assert!(script.contains("CREATE INDEX IF NOT EXISTS idx_jobs_queued_at"));
        assert!(script.contains("CREATE TABLE IF NOT EXISTS metadata"));
    }

    #[tokio::test]
    async fn test_init_local_test_db_creates_database() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Verify database file exists
        assert!(db_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_init_local_test_db_sets_permissions() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Verify permissions (0600)
        #[cfg(unix)]
        {
            let metadata = fs::metadata(&db_path)?;
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o600);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_init_local_test_db_creates_schema() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::sqlite::SqlitePool;

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Connect to verify schema
        let pool = SqlitePool::connect(&format!("sqlite:{db_path_str}?mode=ro")).await?;

        // Verify jobs table exists with correct columns
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM pragma_table_info('jobs') WHERE name IN ('id', 'user', 'benchmark_path', 'status', 'tmux_session_name', 'queued_at', 'started_at', 'completed_at', 'exit_code', 'error_message', 'log_file', 'progress_current', 'progress_total')"
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(result.0, 13); // All 13 columns exist

        // Verify indexes exist
        let indexes: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='jobs'")
                .fetch_all(&pool)
                .await?;

        let index_names: Vec<String> = indexes.into_iter().map(|(name,)| name).collect();
        assert!(index_names.contains(&"idx_jobs_status".to_string()));
        assert!(index_names.contains(&"idx_jobs_user".to_string()));
        assert!(index_names.contains(&"idx_jobs_queued_at".to_string()));

        // Verify metadata table exists
        let metadata_exists: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='metadata'",
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(metadata_exists.0, 1);

        pool.close().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_init_local_test_db_idempotent() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        // Initialize twice
        init_local_test_db(db_path_str).await?;
        init_local_test_db(db_path_str).await?;

        // Should not error (CREATE TABLE IF NOT EXISTS)
        assert!(db_path.exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_init_local_test_db_wal_mode() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::sqlite::SqlitePool;

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Connect and verify WAL mode
        let pool = SqlitePool::connect(&format!("sqlite:{db_path_str}?mode=ro")).await?;

        let result: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await?;

        assert_eq!(result.0.to_lowercase(), "wal");

        pool.close().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_init_local_test_db_busy_timeout() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::sqlite::SqlitePool;

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Connect and verify busy timeout
        let pool = SqlitePool::connect(&format!("sqlite:{db_path_str}?mode=ro")).await?;

        let result: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
            .fetch_one(&pool)
            .await?;

        assert_eq!(result.0, 5000);

        pool.close().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_wrapper_script_compatibility() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::sqlite::SqlitePool;

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Connect to database
        let pool = SqlitePool::connect(&format!("sqlite:{db_path_str}?mode=rwc")).await?;

        // Simulate job insertion (what queue system will do)
        let job_id = "test-job-123";
        sqlx::query(
            "INSERT INTO jobs (id, user, benchmark_path, status, queued_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind(job_id)
        .bind("testuser")
        .bind("/path/to/benchmark.py")
        .bind("queued")
        .execute(&pool)
        .await?;

        // Simulate wrapper script UPDATE: job starting
        sqlx::query(
            "UPDATE jobs SET status=?, started_at=datetime('now'), tmux_session_name=? WHERE id=?",
        )
        .bind("running")
        .bind(format!("solverpilot_testuser_{}", &job_id[..8]))
        .bind(job_id)
        .execute(&pool)
        .await?;

        // Verify running state
        let result: (String, String) =
            sqlx::query_as("SELECT status, started_at FROM jobs WHERE id=?")
                .bind(job_id)
                .fetch_one(&pool)
                .await?;

        assert_eq!(result.0, "running");
        assert!(!result.1.is_empty()); // started_at is set

        // Simulate wrapper script UPDATE: job completed
        sqlx::query(
            "UPDATE jobs SET status=?, completed_at=datetime('now'), exit_code=? WHERE id=?",
        )
        .bind("completed")
        .bind(0)
        .bind(job_id)
        .execute(&pool)
        .await?;

        // Verify completed state
        let result: (String, Option<String>, Option<i64>) =
            sqlx::query_as("SELECT status, completed_at, exit_code FROM jobs WHERE id=?")
                .bind(job_id)
                .fetch_one(&pool)
                .await?;

        assert_eq!(result.0, "completed");
        assert!(result.1.is_some()); // completed_at is set
        assert_eq!(result.2, Some(0)); // exit_code = 0

        pool.close().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_wrapper_script_sql_injection_safety() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::sqlite::SqlitePool;

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Connect to database
        let pool = SqlitePool::connect(&format!("sqlite:{db_path_str}?mode=rwc")).await?;

        // Test with job ID containing single quotes (SQL injection attempt)
        let job_id = "test-job-with-'quote";
        sqlx::query(
            "INSERT INTO jobs (id, user, benchmark_path, status, queued_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind(job_id)
        .bind("testuser")
        .bind("/path/to/benchmark.py")
        .bind("queued")
        .execute(&pool)
        .await?;

        // Using parameterized queries (best practice) - sqlx handles escaping automatically
        sqlx::query("UPDATE jobs SET status=? WHERE id=?")
            .bind("running")
            .bind(job_id) // sqlx handles escaping
            .execute(&pool)
            .await?;

        // Verify update succeeded
        let result: (String,) = sqlx::query_as("SELECT status FROM jobs WHERE id=?")
            .bind(job_id)
            .fetch_one(&pool)
            .await?;

        assert_eq!(result.0, "running");

        pool.close().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_status_check_constraint() -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::sqlite::SqlitePool;

        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_server.db");
        let db_path_str = db_path.to_str().ok_or("Invalid path")?;

        init_local_test_db(db_path_str).await?;

        // Connect to database
        let pool = SqlitePool::connect(&format!("sqlite:{db_path_str}?mode=rwc")).await?;

        // Try to insert job with invalid status (should fail CHECK constraint)
        let result = sqlx::query(
            "INSERT INTO jobs (id, user, benchmark_path, status, queued_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind("test-job-123")
        .bind("testuser")
        .bind("/path/to/benchmark.py")
        .bind("invalid_status") // Invalid status
        .execute(&pool)
        .await;

        assert!(result.is_err()); // Should fail CHECK constraint

        // Valid statuses should work
        let valid_statuses = vec!["queued", "running", "completed", "failed", "killed"];
        for (i, status) in valid_statuses.iter().enumerate() {
            let job_id = format!("test-job-{i}");
            sqlx::query(
                "INSERT INTO jobs (id, user, benchmark_path, status, queued_at)
                 VALUES (?, ?, ?, ?, datetime('now'))",
            )
            .bind(&job_id)
            .bind("testuser")
            .bind("/path/to/benchmark.py")
            .bind(status)
            .execute(&pool)
            .await?;
        }

        pool.close().await;

        Ok(())
    }
}
