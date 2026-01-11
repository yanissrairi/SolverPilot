# Integration Architecture

## Overview

SolverPilot is a **multi-part desktop application** with clear separation between frontend (Svelte UI) and backend (Rust core), communicating via **Tauri IPC**.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Svelte 5)                       │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              UI Components (28+)                      │   │
│  │  - Features: Jobs, Benchmarks, History, SSH, etc.    │   │
│  │  - Reusable UI: Buttons, Modals, Toast, etc.        │   │
│  └─────────────────────┬────────────────────────────────┘   │
│                        ↓                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           State Management (Runes)                    │   │
│  │  - Local: $state, $derived, $effect                  │   │
│  │  - Global: panels, shortcuts, toast stores           │   │
│  └─────────────────────┬────────────────────────────────┘   │
│                        ↓                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │             API Layer (api.ts)                        │   │
│  │  - 40+ typed wrapper functions                       │   │
│  │  - Type-safe interfaces matching Rust                │   │
│  └─────────────────────┬────────────────────────────────┘   │
└────────────────────────┼────────────────────────────────────┘
                         │
                         │ Tauri IPC (JSON)
                         │ invoke('command', {args})
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                   Backend (Rust/Tauri 2)                     │
│  ┌──────────────────────────────────────────────────────┐   │
│  │          Commands Layer (commands.rs)                 │   │
│  │  - 40+ #[tauri::command] functions                    │   │
│  │  - Result<T, String> return types                     │   │
│  └─────────────────────┬────────────────────────────────┘   │
│                        ↓                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │       AppState (Arc<Mutex<T>> shared state)           │   │
│  │  - Config, Database, SSH Manager, Job State          │   │
│  └─────────────────────┬────────────────────────────────┘   │
│                        ↓                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │            Service Modules                            │   │
│  │  - config.rs: Configuration management               │   │
│  │  - db.rs: SQLite operations (SQLx)                   │   │
│  │  - ssh/: Connection pooling, execution, transfer     │   │
│  │  - project.rs: Python project management (uv)        │   │
│  │  - python_deps.rs: Tree-sitter AST analysis          │   │
│  │  - job.rs: Log parsing, progress tracking            │   │
│  └─────────────────────┬────────────────────────────────┘   │
│                        ↓                                      │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           Infrastructure                              │   │
│  │  - SQLite Database (solver-pilot.db)                 │   │
│  │  - SSH Connection Pool (bb8 + russh)                 │   │
│  │  - Local Filesystem (projects/, config.toml)         │   │
│  │  - Remote Server (via SSH: tmux, rsync)              │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Integration Points

### 1. IPC Communication Layer

**Technology**: Tauri IPC with JSON serialization

**Frontend → Backend**:

```typescript
// Frontend (TypeScript)
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<Project>('create_project', {
  name: 'my-solver',
  pythonVersion: '3.12',
});
```

**Backend → Frontend**:

```rust
// Backend (Rust)
#[tauri::command]
async fn create_project(
    state: State<'_, AppState>,
    name: String,
    python_version: String,
) -> Result<Project, String> {
    // Implementation
}
```

**Data Flow**:

1. Frontend calls `invoke()` with command name + args
2. Tauri serializes arguments to JSON
3. Rust deserializes JSON to native types
4. Command executes, returns `Result<T, String>`
5. Tauri serializes result to JSON
6. Frontend receives typed response

**Type Safety**:

- TypeScript interfaces in `types.ts`
- Rust structs with `#[derive(Serialize, Deserialize)]`
- Serde ensures type compatibility

---

### 2. State Synchronization

**Pattern**: Polling + Event-Driven

#### Polling (Job Status)

```typescript
// Frontend polls every 2 seconds
$effect(() => {
  const interval = setInterval(async () => {
    const status = await getJobStatus();
    if (status.is_finished) {
      clearInterval(interval);
    }
  }, 2000);

  return () => clearInterval(interval);
});
```

**Backend streams logs**:

```rust
// Backend tails remote logs, parses progress
async fn get_job_status(state: State<'_, AppState>)
    -> Result<JobStatusResponse, String>
{
    // Tail logs via SSH
    // Parse [x/y] progress
    // Return current state
}
```

#### Request-Response (User Actions)

```typescript
// Frontend initiates action
await queueJobs(['benchmark.py']);
await startNextJob();

// Backend executes, returns result
// No polling needed for one-time operations
```

---

### 3. Database Integration

**Access Pattern**: Backend-Only

- **Frontend**: No direct database access
- **Backend**: SQLx with compile-time checked queries
- **Communication**: All DB operations via Tauri commands

**Example Flow**:

```
Frontend                    Backend
   ↓                           ↓
listProjects()  ──IPC→  list_projects(state)
                            ↓
                        db.lock().await
                            ↓
                        SELECT * FROM projects
                            ↓
                        Vec<Project>
   ↓              ←IPC──    ↓
Display projects
```

**Benefits**:

- Centralized data access
- Single source of truth
- Type-safe queries (SQLx)
- Transaction support

---

### 4. SSH Integration

**Access Pattern**: Backend-Only with Connection Pooling

**Architecture**:

```
Frontend                SSH Module (Backend)
   ↓                           ↓
testSsh()    ──IPC→   SshManager::test_connection()
                            ↓
                    bb8::Pool<SshConnection>
                            ↓
                    Get connection from pool
                            ↓
                    russh::client::connect()
                            ↓
                    Execute command
                            ↓
                    Return connection to pool
```

**Connection Pool** (bb8):

- **Max Connections**: Configurable (default: 10)
- **Idle Timeout**: Auto-close unused connections
- **Health Checks**: Periodic connection validation
- **Reuse**: Avoid SSH handshake overhead

**Operations**:

1. **Authentication** (`ssh/auth.rs`):
   - Load SSH key from filesystem
   - Support passphrase-protected keys
   - SSH agent integration

2. **Command Execution** (`ssh/executor.rs`):
   - Execute commands via SSH
   - tmux session management for jobs
   - Streaming output capture

3. **File Transfer** (`ssh/transfer.rs`):
   - rsync-based synchronization
   - Incremental updates
   - Dependency-only sync option

---

### 5. File System Integration

**Shared Access**: Both frontend and backend

#### Config File (config.toml)

- **Write**: Backend only (`save_config`)
- **Read**: Backend only (`load_config`)
- **Frontend Access**: Via IPC commands
- **Location**: Platform-specific config directory

#### Project Files

- **Write**: Backend only (project creation, uv operations)
- **Read**: Backend for dependency analysis
- **Frontend Access**: Via file picker dialog + IPC
- **Location**: `projects/{name}/`

#### Database (solver-pilot.db)

- **Access**: Backend only (SQLx)
- **Location**: Configured in config.toml
- **Frontend**: All access via IPC commands

---

## Data Flow Examples

### Example 1: Creating and Running a Job

```
┌─────────────┐
│  Frontend   │
└──────┬──────┘
       │
       │ 1. User clicks "Run Benchmark"
       ↓
   queueJobs(['bench.py'])
       │
       │ IPC invoke('queue_jobs', {benchmarkNames: ['bench.py']})
       ↓
┌─────────────┐
│  Backend    │
└──────┬──────┘
       │
       │ 2. Lock DB, insert job with status='pending'
       ↓
   db::insert_job()
       │
       │ 3. Return Job[]
       ↓
┌─────────────┐
│  Frontend   │
└──────┬──────┘
       │
       │ 4. User clicks "Start"
       ↓
   startNextJob()
       │
       │ IPC invoke('start_next_job')
       ↓
┌─────────────┐
│  Backend    │
└──────┬──────┘
       │
       │ 5. Find first pending job
       ├─→ db::find_pending_job()
       │
       │ 6. Update status to 'running'
       ├─→ db::update_job_status()
       │
       │ 7. Sync code to remote
       ├─→ ssh::transfer::rsync()
       │
       │ 8. Start tmux session
       ├─→ ssh::executor::start_job()
       │
       │ 9. Return Job
       ↓
┌─────────────┐
│  Frontend   │
└──────┬──────┘
       │
       │ 10. Start polling loop (every 2s)
       ↓
   $effect(() => {
     setInterval(async () => {
       const status = await getJobStatus();
       // Update UI
     }, 2000);
   })
       │
       │ IPC invoke('get_job_status')
       ↓
┌─────────────┐
│  Backend    │
└──────┬──────┘
       │
       │ 11. Tail remote logs
       ├─→ ssh::executor::tail_logs()
       │
       │ 12. Parse progress [x/y]
       ├─→ job::parse_progress()
       │
       │ 13. Check for completion
       ├─→ job::is_finished()
       │
       │ 14. Return JobStatusResponse
       ↓
┌─────────────┐
│  Frontend   │
└──────┬──────┘
       │
       │ 15. Update UI with progress
       │     Show logs, elapsed time, etc.
       ↓
   (Poll continues until job finished)
```

### Example 2: Dependency Analysis

```
Frontend: getBenchmarkDependencies('benchmark.py')
   ↓ IPC
Backend: get_benchmark_dependencies(path)
   ↓
Read Python file from filesystem
   ↓
python_deps::analyze_dependencies()
   ↓ tree-sitter AST parsing
Extract imports:
  - from foo import bar  → Local dependency
  - import numpy         → External package
   ↓
Build dependency tree (recursive)
   ↓
Check pyproject.toml for external packages
   ↓
Return DependencyAnalysis {
  local_files: [...],
  external_packages: [...]
}
   ↓ IPC
Frontend: Display dependency tree in UI
```

---

## Communication Patterns

### 1. Command Pattern

**Use Case**: User-initiated actions

**Example**: Create project, add benchmark, sync code

**Flow**: Request → Execute → Response

**Error Handling**: `Result<T, String>` with user-friendly messages

### 2. Polling Pattern

**Use Case**: Real-time updates (job monitoring)

**Interval**: 2 seconds (configurable in config.toml)

**Optimization**: Backend caches logs, incremental reads

**Termination**: `is_finished` flag stops polling

### 3. One-Shot Pattern

**Use Case**: Configuration, database queries

**Example**: Load config, list projects, load history

**Flow**: Request → Query → Response

**Caching**: Frontend caches where appropriate

---

## Security Boundaries

### Credential Protection

- **SSH Keys**: Never exposed to frontend
- **Passphrases**: Handled in backend, zeroized after use
- **Config**: Backend reads, frontend gets sanitized view

### Input Validation

- **Frontend**: UI constraints (required fields, formats)
- **Backend**: Explicit validation at command boundary
- **Database**: Parameterized queries (SQL injection prevention)

### File Access

- **Frontend**: File picker dialog (user-controlled)
- **Backend**: Path sanitization, no arbitrary access
- **Remote**: Scoped to configured base directory

---

## Performance Optimizations

### 1. Connection Pooling

- **SSH connections** reused via bb8
- Reduces handshake overhead (~500ms per connection)
- Configurable pool size

### 2. Incremental Updates

- **Log tailing**: Read only new lines
- **File sync**: rsync sends only changes
- **Progress**: Parse on-the-fly, no full log storage

### 3. Async Everything

- **Backend**: Tokio async runtime
- **Frontend**: Promise-based API
- **No blocking**: All I/O is non-blocking

### 4. Smart Polling

- **Only when needed**: Poll only during running jobs
- **Backoff**: Could implement exponential backoff (future)
- **Cancellation**: Cleanup on component unmount

---

## Error Propagation

### Error Flow

```
Backend Error
   ↓
Convert to String (format!("Error: {e}"))
   ↓
Result<T, String>
   ↓
Tauri IPC serialization
   ↓
Frontend catch block
   ↓
Toast notification (user-visible)
```

### Error Types

1. **Database errors**: "Failed to load projects"
2. **SSH errors**: "Connection failed: ..."
3. **File errors**: "Benchmark not found: ..."
4. **Validation errors**: "Invalid project name"

### User Experience

- All errors shown via toast notifications
- Error type determines toast color (error/warning)
- Auto-dismiss after 5-8 seconds
- Critical errors persist until dismissed

---

## Summary

**Integration Type**: Tauri IPC with JSON serialization

**Communication Patterns**:

- Command pattern (user actions)
- Polling pattern (real-time updates)
- One-shot pattern (queries)

**Data Flow**: Frontend ↔ IPC ↔ Backend ↔ Infrastructure

**Key Integration Points**:

1. **IPC Commands**: 40+ typed commands
2. **State Sync**: Polling for jobs, request-response for actions
3. **Database**: Backend-only access via SQLx
4. **SSH**: Connection pooling with russh + bb8
5. **File System**: Scoped access, sanitized paths

**Performance**:

- Connection pooling
- Incremental updates
- Async I/O
- Smart polling

**Security**:

- Credential protection
- Input validation
- Parameterized queries
- Scoped file access
