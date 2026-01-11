# Data Models - Backend

## Database Overview

**Database**: SQLite (via SQLx)
**Location**: `solver-pilot.db` (configured path)
**Features**:

- Foreign key constraints enabled
- Cascade deletes
- Compile-time checked queries

---

## Database Schema

### Table: `projects`

Python project environments managed via `uv`.

| Column           | Type    | Constraints               | Description                           |
| ---------------- | ------- | ------------------------- | ------------------------------------- |
| `id`             | INTEGER | PRIMARY KEY AUTOINCREMENT | Unique project ID                     |
| `name`           | TEXT    | NOT NULL, UNIQUE          | Project name (must be unique)         |
| `python_version` | TEXT    | NOT NULL, DEFAULT '3.12'  | Python version (e.g., "3.12", "3.11") |
| `created_at`     | TEXT    | NOT NULL                  | ISO 8601 timestamp                    |
| `updated_at`     | TEXT    | NOT NULL                  | ISO 8601 timestamp                    |

**Indexes**: Primary key on `id`, unique index on `name`

---

### Table: `benchmarks`

Python benchmark files associated with projects.

| Column       | Type    | Constraints                                            | Description                       |
| ------------ | ------- | ------------------------------------------------------ | --------------------------------- |
| `id`         | INTEGER | PRIMARY KEY AUTOINCREMENT                              | Unique benchmark ID               |
| `project_id` | INTEGER | NOT NULL, FOREIGN KEY → projects(id) ON DELETE CASCADE | Parent project                    |
| `name`       | TEXT    | NOT NULL                                               | Benchmark display name (filename) |
| `path`       | TEXT    | NOT NULL                                               | Absolute file path to `.py` file  |
| `created_at` | TEXT    | NOT NULL                                               | ISO 8601 timestamp                |

**Indexes**:

- Primary key on `id`
- Unique constraint on `(project_id, path)`

**Foreign Keys**:

- `project_id` → `projects(id)` with CASCADE delete

---

### Table: `jobs`

Benchmark execution jobs with status tracking.

| Column             | Type    | Constraints                                                                | Description                                  |
| ------------------ | ------- | -------------------------------------------------------------------------- | -------------------------------------------- |
| `id`               | INTEGER | PRIMARY KEY AUTOINCREMENT                                                  | Unique job ID                                |
| `project_id`       | INTEGER | FOREIGN KEY → projects(id), NULL allowed                                   | Associated project (legacy jobs may be NULL) |
| `benchmark_name`   | TEXT    | NOT NULL                                                                   | Benchmark filename to execute                |
| `status`           | TEXT    | NOT NULL, CHECK IN ('pending', 'running', 'completed', 'failed', 'killed') | Current job status                           |
| `created_at`       | TEXT    | NOT NULL                                                                   | ISO 8601 timestamp                           |
| `started_at`       | TEXT    | NULL                                                                       | ISO 8601 timestamp when job started          |
| `finished_at`      | TEXT    | NULL                                                                       | ISO 8601 timestamp when job finished         |
| `progress_current` | INTEGER | DEFAULT 0                                                                  | Current progress count (parsed from logs)    |
| `progress_total`   | INTEGER | DEFAULT 0                                                                  | Total progress count (parsed from logs)      |
| `results_path`     | TEXT    | NULL                                                                       | Remote path to results file                  |
| `error_message`    | TEXT    | NULL                                                                       | Error message if failed                      |
| `log_content`      | TEXT    | NULL                                                                       | Captured log output                          |

**Indexes**: Primary key on `id`

**Foreign Keys**:

- `project_id` → `projects(id)` (no cascade, allows NULL)

**Status Values**:

- `pending`: Queued, not yet started
- `running`: Currently executing on remote server
- `completed`: Finished successfully
- `failed`: Finished with error
- `killed`: Manually terminated

---

## Rust Data Models

### AppConfig

Configuration loaded from `config.toml`.

```rust
pub struct AppConfig {
    pub ssh: SshConfig,
    pub remote: RemoteConfig,
    pub polling: PollingConfig,
    pub gurobi: GurobiConfig,
    pub tools: ToolsConfig,
}

pub struct SshConfig {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub key_path: String,
}

pub struct RemoteConfig {
    pub remote_base: String,  // Base directory on remote server
}

pub struct PollingConfig {
    pub interval_seconds: u64,  // Job status polling interval
}

pub struct GurobiConfig {
    pub home: String,           // GUROBI_HOME path
    pub license_file: String,   // License file path
}

pub struct ToolsConfig {
    pub uv_path: String,  // Path to uv executable
}
```

---

### Project

Python project with managed environment.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub python_version: String,
    pub created_at: String,     // ISO 8601
    pub updated_at: String,     // ISO 8601
}
```

**Directory Structure** (per project):

```
projects/{name}/
├── .python-version        # Python version file
├── pyproject.toml         # uv project manifest
├── uv.lock                # uv lockfile
├── .venv/                 # Virtual environment
└── benchmarks/            # Benchmark .py files
```

---

### Benchmark

Reference to a Python benchmark file.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    pub id: i64,
    pub project_id: i64,
    pub name: String,           // Display name
    pub path: String,           // Absolute path to .py file
    pub created_at: String,     // ISO 8601
}
```

---

### Job

Benchmark execution job.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub project_id: Option<i64>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Killed,
}
```

---

### JobStatusResponse

Enriched job status for frontend.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub job: Option<Job>,
    pub logs: String,
    pub progress: f32,           // 0.0 to 1.0
    pub progress_text: String,   // e.g., "[12/50]"
    pub elapsed_seconds: u64,
    pub is_finished: bool,
    pub error: Option<String>,
}
```

---

### SyncStatus

Code synchronization status.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SyncStatus {
    Checking,
    UpToDate,
    Modified { count: usize, files: Vec<String> },
    Syncing,
    Error { message: String },
}
```

---

### SshKeyStatus

SSH key agent status.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SshKeyStatus {
    InAgent,
    NeedsPassphrase { key_path: String },
    NoKey { expected_path: String },
    NoAgent,
}
```

---

### DependencyAnalysis

Python dependency tree analysis (via tree-sitter).

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub root: String,
    pub local_files: Vec<LocalDependency>,
    pub external_packages: Vec<ExternalPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDependency {
    pub module_name: String,
    pub file_path: String,
    pub exists: bool,
    pub children: Vec<LocalDependency>,  // Recursive tree
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPackage {
    pub name: String,
    pub in_pyproject: bool,  // Whether listed in pyproject.toml
}
```

---

## AppState (Global State)

Thread-safe application state using `Arc<Mutex<T>>`.

```rust
pub struct AppState {
    pub config: Arc<Mutex<Option<AppConfig>>>,
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub ssh_manager: Arc<Mutex<Option<SshManager>>>,
    pub current_job_id: Arc<Mutex<Option<i64>>>,
    pub job_start_time: Arc<Mutex<Option<std::time::Instant>>>,
    pub current_project_id: Arc<Mutex<Option<i64>>>,
}
```

**Lifecycle**:

1. Initialized with `None` values on startup
2. Config loaded from `config.toml`
3. Database initialized at configured path
4. SSH manager created from config
5. Current project/job tracked as user interacts

---

## Database Operations

### CRUD Patterns

**Insert**:

```rust
pub async fn insert_project(
    pool: &SqlitePool,
    name: &str,
    python_version: &str,
) -> Result<i64, String>
```

**Read**:

```rust
pub async fn get_project(pool: &SqlitePool, id: i64)
    -> Result<Option<Project>, String>

pub async fn list_projects(pool: &SqlitePool)
    -> Result<Vec<Project>, String>
```

**Update**:

```rust
pub async fn update_project_python_version(
    pool: &SqlitePool,
    id: i64,
    version: &str,
) -> Result<(), String>
```

**Delete**:

```rust
pub async fn delete_project(pool: &SqlitePool, id: i64)
    -> Result<(), String>
```

---

## Relationships

### Entity Relationships

```
projects (1) ──< (many) benchmarks
    │
    │ (optional)
    ↓
   jobs (many)
```

**Cascade Behavior**:

- Delete project → cascade delete all benchmarks
- Delete project → jobs remain (project_id becomes NULL)

---

## Data Flow

### Job Execution Flow

```
1. Frontend: queueJobs(["benchmark.py"])
   ↓
2. Backend: Insert Job with status='pending'
   ↓
3. Frontend: startNextJob()
   ↓
4. Backend:
   - Find first pending job
   - Set status='running'
   - rsync project files
   - Start tmux session on remote
   - Update started_at
   ↓
5. Frontend: Poll getJobStatus() every 2s
   ↓
6. Backend:
   - Tail remote logs
   - Parse progress: [current/total]
   - Update progress_current, progress_total
   - Detect completion patterns
   ↓
7. Completion:
   - Update status (completed/failed/killed)
   - Set finished_at
   - Store results_path or error_message
```

---

## Summary

**Tables**: 3 (projects, benchmarks, jobs)
**Data Models**: 11 Rust structs
**State**: Thread-safe Arc<Mutex<T>> wrappers
**Serialization**: Serde with JSON for IPC
**Queries**: SQLx with compile-time checking
**Timestamps**: ISO 8601 via chrono
**Foreign Keys**: Enabled with cascade deletes
