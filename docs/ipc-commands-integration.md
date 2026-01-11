# IPC Commands - Integration Layer

## Overview

SolverPilot uses **Tauri IPC (Inter-Process Communication)** to bridge the Svelte frontend with the Rust backend. The frontend invokes Rust functions via `@tauri-apps/api`, and the backend exposes functions as Tauri commands.

**Total Commands**: 40+

**API Layer**: `src/lib/api.ts` wraps all Tauri invocations with TypeScript type safety.

---

## Command Categories

### 1. Configuration (5 commands)

Commands for managing application configuration (`config.toml`).

| Command               | Frontend API          | Return Type | Purpose                                    |
| --------------------- | --------------------- | ----------- | ------------------------------------------ |
| `check_config_exists` | `checkConfigExists()` | `boolean`   | Check if config file exists                |
| `get_config_path`     | `getConfigPath()`     | `string`    | Get config file path                       |
| `load_config`         | `loadConfig()`        | `AppConfig` | Load configuration from file               |
| `save_config`         | `saveConfig(config)`  | `void`      | Save configuration to file                 |
| `init_db`             | _(internal)_          | -           | Initialize database (called automatically) |

**Example**:

```typescript
const exists = await checkConfigExists();
if (!exists) {
  const config = {
    /* default config */
  };
  await saveConfig(config);
}
const loaded = await loadConfig();
```

---

### 2. SSH Management (6 commands)

Commands for SSH connection and authentication.

| Command                | Frontend API                 | Return Type    | Purpose                                           |
| ---------------------- | ---------------------------- | -------------- | ------------------------------------------------- |
| `init_ssh`             | `initSsh()`                  | `string`       | Initialize SSH connection with connection pooling |
| `close_ssh`            | `closeSsh()`                 | `void`         | Close SSH connection and cleanup pool             |
| `test_ssh`             | `testSsh()`                  | `boolean`      | Test SSH connection (using pooled connection)     |
| `test_ssh_direct`      | `testSshDirect(passphrase?)` | `void`         | Test SSH without using pool (for setup wizard)    |
| `check_ssh_key_status` | `checkSshKeyStatus()`        | `SshKeyStatus` | Check if SSH key is in agent                      |
| `add_ssh_key`          | `addSshKey(passphrase)`      | `void`         | Add SSH key to agent with passphrase              |

**SSH Key Status**:

```typescript
type SshKeyStatus =
  | { type: 'InAgent' }
  | { type: 'NeedsPassphrase'; data: { key_path: string } }
  | { type: 'NoKey'; data: { expected_path: string } }
  | { type: 'NoAgent' };
```

**Example**:

```typescript
const status = await checkSshKeyStatus();
if (status.type === 'NeedsPassphrase') {
  const passphrase = await promptUser();
  await addSshKey(passphrase);
}
await initSsh();
```

---

### 3. Code Synchronization (3 commands)

Commands for syncing code to remote server via rsync.

| Command               | Frontend API              | Return Type  | Purpose                                    |
| --------------------- | ------------------------- | ------------ | ------------------------------------------ |
| `check_sync_status`   | `checkSyncStatus()`       | `SyncStatus` | Check if local files differ from remote    |
| `sync_code`           | `syncCode()`              | `void`       | Sync entire project to remote              |
| `sync_benchmark_deps` | `syncBenchmarkDeps(path)` | `number`     | Sync only dependency files for a benchmark |

**Sync Status**:

```typescript
type SyncStatus =
  | { type: 'Checking' }
  | { type: 'UpToDate' }
  | { type: 'Modified'; data: { count: number; files: string[] } }
  | { type: 'Syncing' }
  | { type: 'Error'; data: { message: string } };
```

**Example**:

```typescript
const status = await checkSyncStatus();
if (status.type === 'Modified') {
  console.log(`${status.data.count} files changed`);
  await syncCode();
}
```

---

### 4. Project Management (5 commands)

Commands for managing Python project environments via `uv`.

| Command              | Frontend API                         | Return Type       | Purpose                                    |
| -------------------- | ------------------------------------ | ----------------- | ------------------------------------------ |
| `list_projects`      | `listProjects()`                     | `Project[]`       | List all projects                          |
| `create_project`     | `createProject(name, pythonVersion)` | `Project`         | Create new project with Python environment |
| `delete_project`     | `deleteProject(projectId)`           | `void`            | Delete project and its directory           |
| `set_active_project` | `setActiveProject(projectId)`        | `Project`         | Set active project in state                |
| `get_active_project` | `getActiveProject()`                 | `Project \| null` | Get currently active project               |

**Example**:

```typescript
const projects = await listProjects();
const newProject = await createProject('my-solver', '3.12');
await setActiveProject(newProject.id);
```

---

### 5. Python Version Management (2 commands)

Commands for managing Python versions via `uv`.

| Command                      | Frontend API                       | Return Type | Purpose                                 |
| ---------------------------- | ---------------------------------- | ----------- | --------------------------------------- |
| `list_python_versions`       | `listPythonVersions()`             | `string[]`  | List available Python versions from uv  |
| `set_project_python_version` | `setProjectPythonVersion(version)` | `void`      | Change Python version of active project |

**Example**:

```typescript
const versions = await listPythonVersions();
// ["3.12", "3.11", "3.10", ...]
await setProjectPythonVersion('3.11');
```

---

### 6. Benchmark Management (4 commands)

Commands for managing benchmark files in projects.

| Command                         | Frontend API                              | Return Type          | Purpose                                     |
| ------------------------------- | ----------------------------------------- | -------------------- | ------------------------------------------- |
| `list_project_benchmarks`       | `listProjectBenchmarks()`                 | `Benchmark[]`        | List benchmarks in active project           |
| `add_benchmark_to_project`      | `addBenchmarkToProject(filePath)`         | `Benchmark`          | Add benchmark file to project               |
| `remove_benchmark_from_project` | `removeBenchmarkFromProject(benchmarkId)` | `void`               | Remove benchmark from project               |
| `get_benchmark_dependencies`    | `getBenchmarkDependencies(path)`          | `DependencyAnalysis` | Analyze Python dependencies via tree-sitter |

**Dependency Analysis**:

```typescript
interface DependencyAnalysis {
  root: string;
  local_files: LocalDependency[]; // Recursive tree of local imports
  external_packages: ExternalPackage[]; // External package imports
}
```

**Example**:

```typescript
const benchmarks = await listProjectBenchmarks();
const deps = await getBenchmarkDependencies(benchmarks[0].path);
console.log('External packages:', deps.external_packages);
```

---

### 7. Dependency Management (5 commands)

Commands for managing Python dependencies via `uv`.

| Command                       | Frontend API                           | Return Type | Purpose                               |
| ----------------------------- | -------------------------------------- | ----------- | ------------------------------------- |
| `list_project_dependencies`   | `listProjectDependencies()`            | `string[]`  | List dependencies from pyproject.toml |
| `add_project_dependency`      | `addProjectDependency(packageName)`    | `string`    | Add dependency via `uv add`           |
| `remove_project_dependency`   | `removeProjectDependency(packageName)` | `string`    | Remove dependency via `uv remove`     |
| `update_project_dependencies` | `updateProjectDependencies()`          | `string`    | Update all dependencies via `uv sync` |
| `sync_project_environment`    | `syncProjectEnvironment()`             | `string`    | Sync environment via `uv sync`        |

**Example**:

```typescript
await addProjectDependency('numpy');
await addProjectDependency('gurobipy');
await syncProjectEnvironment();

const deps = await listProjectDependencies();
// ["numpy", "gurobipy", ...]
```

---

### 8. Job Execution (5 commands)

Commands for managing benchmark job execution on remote server.

| Command          | Frontend API                | Return Type         | Purpose                               |
| ---------------- | --------------------------- | ------------------- | ------------------------------------- |
| `queue_jobs`     | `queueJobs(benchmarkNames)` | `Job[]`             | Queue benchmarks for execution        |
| `start_next_job` | `startNextJob()`            | `Job \| null`       | Start next pending job                |
| `stop_job`       | `stopJob()`                 | `void`              | Gracefully stop running job (SIGTERM) |
| `kill_job`       | `killJob()`                 | `void`              | Force kill running job (SIGKILL)      |
| `get_job_status` | `getJobStatus()`            | `JobStatusResponse` | Get current job status with progress  |

**Job Status Response**:

```typescript
interface JobStatusResponse {
  job: Job | null;
  logs: string;
  progress: number; // 0.0 to 1.0
  progress_text: string; // e.g., "[15/50]"
  elapsed_seconds: number;
  is_finished: boolean;
  error: string | null;
}
```

**Execution Flow**:

```typescript
// 1. Queue jobs
const jobs = await queueJobs(['benchmark1.py', 'benchmark2.py']);

// 2. Start first job
const job = await startNextJob();

// 3. Poll for status
const interval = setInterval(async () => {
  const status = await getJobStatus();
  console.log(`Progress: ${status.progress_text}`);

  if (status.is_finished) {
    clearInterval(interval);
  }
}, 2000);

// 4. Stop if needed
await stopJob();
```

---

### 9. Job Logs (1 command)

Commands for retrieving job logs.

| Command        | Frontend API        | Return Type | Purpose                      |
| -------------- | ------------------- | ----------- | ---------------------------- |
| `get_job_logs` | `getJobLogs(lines)` | `string`    | Get last N lines of job logs |

**Example**:

```typescript
const logs = await getJobLogs(100); // Last 100 lines
console.log(logs);
```

---

### 10. Job History (2 commands)

Commands for managing job history.

| Command        | Frontend API         | Return Type | Purpose                              |
| -------------- | -------------------- | ----------- | ------------------------------------ |
| `load_history` | `loadHistory(limit)` | `Job[]`     | Load job history (most recent first) |
| `delete_job`   | `deleteJob(jobId)`   | `void`      | Delete job from history              |

**Example**:

```typescript
const history = await loadHistory(50); // Last 50 jobs
const completedJobs = history.filter(j => j.status === 'completed');

// Delete old jobs
await deleteJob(oldJobId);
```

---

## Command Implementation Pattern

### Backend (Rust)

Commands are defined in `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?
        .clone();

    crate::db::list_projects(&db).await
}
```

**Pattern**:

1. Access shared state via `State<'_, AppState>`
2. Lock mutex to access resources
3. Perform operation
4. Return `Result<T, String>` (errors as strings)

### Frontend (TypeScript)

Commands are wrapped in `src/lib/api.ts`:

```typescript
export async function listProjects(): Promise<Project[]> {
  return invoke('list_projects');
}
```

**Pattern**:

1. Type-safe wrapper function
2. Call `invoke()` with command name
3. Arguments passed as object: `invoke('cmd', { arg1, arg2 })`
4. TypeScript enforces return types

---

## Error Handling

### Backend

All commands return `Result<T, String>`:

```rust
#[tauri::command]
async fn create_project(
    state: State<'_, AppState>,
    name: String,
    python_version: String,
) -> Result<Project, String> {
    let db = state.db.lock().await
        .as_ref()
        .ok_or("Database not initialized")?;

    // Create project directory
    let project_path = get_project_path(&name)?;
    std::fs::create_dir_all(&project_path)
        .map_err(|e| format!("Failed to create directory: {e}"))?;

    // Insert into database
    let id = db::insert_project(db, &name, &python_version).await?;

    // Return project
    db::get_project(db, id).await?
        .ok_or_else(|| "Project not found after creation".to_string())
}
```

### Frontend

Catch errors and display to user:

```typescript
try {
  const project = await createProject(name, version);
  toast.success('Project created!');
} catch (error) {
  toast.error(`Failed: ${error}`);
}
```

---

## Performance Considerations

### Connection Pooling

- SSH connections reused via bb8 pool
- Reduces connection overhead
- Max connections configured per use case

### Polling Strategy

- Frontend polls job status every 2s
- Backend tails logs incrementally
- Progress parsed from log patterns

### Async Operations

- All commands are async (Tokio runtime)
- Non-blocking I/O throughout
- Database queries run concurrently

---

## Security

### Credential Handling

- SSH keys never sent to frontend
- Passphrases stored only in memory (zeroized)
- Config file contains only key paths, not keys themselves

### Input Validation

- All user inputs validated at command boundary
- SQL injection prevented by SQLx parameterized queries
- Path traversal checks for file operations

### Command Authorization

- No privilege escalation
- Commands run with app permissions
- Remote execution limited to configured user/host

---

## Summary

**Command Count**: 40+ Tauri commands
**Categories**: 10 functional groups
**Pattern**: Request-response with JSON serialization
**Error Handling**: Result<T, String> with user-friendly messages
**Type Safety**: TypeScript interfaces match Rust structs
**Performance**: Connection pooling, async I/O, incremental updates
**Security**: Credential protection, input validation, no privilege escalation
