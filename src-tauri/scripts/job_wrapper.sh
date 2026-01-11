#!/bin/bash
# job_wrapper.sh - Robust job state capture wrapper
# Purpose: Guarantees 99.99% state capture for remote SSH/tmux jobs
# Usage: job_wrapper.sh <job_id> <command> [args...]
#
# Exit Codes:
#   0   - Job completed successfully
#   1   - Job failed or wrapper error
#   130 - Job killed by SIGINT (Ctrl+C)
#   143 - Job killed by SIGTERM
#
# Limitation: trap EXIT does NOT run on SIGKILL (kill -9) - expected edge case
# that will be detected by reconciliation logic (Epic 3)

set -euo pipefail

# Extract job ID and command
JOB_ID="$1"
shift

# Escape single quotes for SQL safety
JOB_ID_SQL="${JOB_ID//\'/\'\'}"

# Environment setup
USER="${USER:-$(whoami)}"
BASE_DIR="$HOME/.solverpilot-server"
SERVER_DB="$BASE_DIR/server.db"
STATE_FILE="$BASE_DIR/jobs/$JOB_ID.status"
LOCK_FILE="$BASE_DIR/locks/$JOB_ID.lock"

# Store started_at for inclusion in completion state
STARTED_AT=""

# Create necessary directories
mkdir -p "$BASE_DIR"/{jobs,locks}

# Acquire exclusive lock for atomic operations
exec 200>"$LOCK_FILE"
if ! flock -x 200; then
    echo "ERROR: Could not acquire lock for job $JOB_ID" >&2
    exit 1
fi

# Cleanup function - called on EXIT (guaranteed unless SIGKILL)
cleanup() {
    local exit_code=$?
    local status="completed"
    [[ $exit_code -ne 0 ]] && status="failed"

    # Write to SQLite (primary source of truth)
    if command -v sqlite3 &>/dev/null; then
        sqlite3 "$SERVER_DB" <<SQL 2>/dev/null || echo "WARNING: Failed to update SQLite, state file written" >&2
UPDATE jobs
SET status='$status',
    completed_at=datetime('now'),
    exit_code=$exit_code
WHERE id='$JOB_ID_SQL';
SQL
    fi

    # Write to state file (fallback + redundancy) - preserve started_at
    cat >"$STATE_FILE" <<JSON
{
  "id": "$JOB_ID",
  "status": "$status",
  "exit_code": $exit_code,
  "started_at": "$STARTED_AT",
  "completed_at": "$(date -Iseconds)",
  "user": "$USER"
}
JSON

    flock -u 200
}

# Signal handlers - ensure proper exit codes for killed jobs
trap 'exit 143' TERM
trap 'exit 130' INT
trap cleanup EXIT

# Capture start time
STARTED_AT="$(date -Iseconds)"

# Update: Job starting (write to both SQLite and state file)
if command -v sqlite3 &>/dev/null; then
    sqlite3 "$SERVER_DB" <<SQL 2>/dev/null || true
UPDATE jobs
SET status='running',
    started_at=datetime('now'),
    tmux_session_name='solverpilot_${USER}_${JOB_ID_SQL:0:8}'
WHERE id='$JOB_ID_SQL';
SQL
fi

cat >"$STATE_FILE" <<JSON
{
  "id": "$JOB_ID",
  "status": "running",
  "started_at": "$STARTED_AT",
  "user": "$USER"
}
JSON

# Execute the actual job - exit code captured by trap EXIT
"$@"
