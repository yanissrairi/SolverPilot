-- Server-side database schema for SolverPilot remote job execution
-- Location: ~/.solverpilot-server/server.db (on remote SSH server)
-- Purpose: Track actual job execution state (updated by wrapper script)

-- Enable performance features
PRAGMA journal_mode = WAL;       -- Write-Ahead Logging for concurrent reads/writes
PRAGMA busy_timeout = 5000;      -- Wait up to 5 seconds on lock contention
PRAGMA foreign_keys = ON;        -- Enable FK constraints (future-proofing)

-- Jobs table (execution reality)
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,                     -- UUID from client (matches client DB job ID)
    user TEXT NOT NULL DEFAULT 'default',    -- SSH user running the job
    benchmark_path TEXT NOT NULL,            -- Remote path to benchmark .py file
    status TEXT NOT NULL CHECK(status IN ('queued', 'running', 'completed', 'failed', 'killed')),
    tmux_session_name TEXT UNIQUE,           -- solverpilot_<user>_<job_id_short>
    queued_at TEXT NOT NULL,                 -- ISO 8601 timestamp when job added to queue
    started_at TEXT,                         -- ISO 8601 timestamp when wrapper started job
    completed_at TEXT,                       -- ISO 8601 timestamp when job finished
    exit_code INTEGER,                       -- Process exit code (0=success, >0=failure)
    error_message TEXT,                      -- Last 20 lines of log if failed
    log_file TEXT,                           -- Path to full log file on server
    progress_current INTEGER,                -- Parsed [x/y] progress (x)
    progress_total INTEGER                   -- Parsed [x/y] progress (y)
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);          -- Filter by status (running, completed, etc.)
CREATE INDEX IF NOT EXISTS idx_jobs_user ON jobs(user);              -- Filter by user (multi-user support)
CREATE INDEX IF NOT EXISTS idx_jobs_queued_at ON jobs(queued_at);    -- Order by queue time (FIFO)

-- Metadata table (future extension point)
CREATE TABLE IF NOT EXISTS metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Store wrapper version for debugging
INSERT OR REPLACE INTO metadata (key, value, updated_at)
VALUES ('wrapper_version', '1.0.0', datetime('now'));
