# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development (hot-reload)
bun run tauri dev

# Production build
bun run tauri build

# Frontend type-check
bun run check

# Lint & format
bun run lint          # ESLint fix
bun run format        # Prettier
bun run quality       # All checks (lint + format + type-check)

# Rust checks (from src-tauri/)
cargo clippy
cargo fmt
cargo deny check
```

## Architecture Overview

SolverPilot is a Tauri 2 desktop app for running Python optimization benchmarks on remote servers via SSH.

```
┌─────────────────┐     JSON/IPC      ┌──────────────────────┐
│  Svelte 5 UI    │ ◄──────────────► │  Rust Backend        │
│  (src/)         │                   │  (src-tauri/src/)    │
└─────────────────┘                   └──────────┬───────────┘
                                                 │
                    ┌────────────────────────────┼────────────────────┐
                    │                            │                    │
                    ▼                            ▼                    ▼
              config.toml                   SQLite DB            Remote Server
              (user config)                 (jobs, projects)     (SSH + tmux)
```

### Backend (Rust - src-tauri/src/)

| Module | Purpose |
|--------|---------|
| `lib.rs` | Tauri setup, registers 40+ commands |
| `state.rs` | Thread-safe `AppState` with `Arc<Mutex<T>>` |
| `commands.rs` | All Tauri commands (config, ssh, sync, projects, jobs) |
| `config.rs` | Loads `config.toml`, path helpers |
| `db.rs` | SQLite via sqlx (projects, benchmarks, jobs tables) |
| `ssh.rs` | SSH control socket, rsync, tmux job management |
| `project.rs` | Python project management via `uv` |
| `python_deps.rs` | Tree-sitter Python AST analysis for imports |
| `job.rs` | Log parsing, progress extraction `[x/y]` |

### Frontend (Svelte 5 - src/)

| Directory | Purpose |
|-----------|---------|
| `lib/features/` | Domain components (benchmarks, jobs, history, ssh) |
| `lib/layout/` | MainLayout (3-panel), Header, ResizablePanel |
| `lib/stores/` | Svelte 5 runes stores (panels, shortcuts, toast) |
| `lib/ui/` | Reusable components (Button, Modal, Badge, Toast...) |
| `lib/utils/` | Utilities (focus-trap, keyboard) |
| `lib/api.ts` | Tauri invoke wrappers |
| `lib/types.ts` | TypeScript interfaces |

## Key Patterns

### Rust Backend

**Error handling** - Always `Result<T, String>`, never panic:
```rust
#[tauri::command]
async fn my_command(state: State<'_, AppState>) -> Result<T, String> {
    let config = state.config.lock().await
        .as_ref()
        .ok_or("Config not loaded")?;
    // ...
}
```

**State access** - Lock then clone/use:
```rust
let db = state.db.lock().await
    .as_ref()
    .ok_or("Database not initialized")?
    .clone();
```

### Svelte 5 Frontend

**Runes** (not legacy stores):
```typescript
let items = $state<Item[]>([])           // reactive state
let count = $derived(items.length)        // computed
$effect(() => { /* side effect */ })      // auto-cleanup
```

**Component props**:
```svelte
<script lang="ts">
  import type { Snippet } from 'svelte'
  interface Props { title: string; children?: Snippet }
  const { title, children }: Props = $props()
</script>
```

**API calls**:
```typescript
import * as api from '$lib/api'
const result = await api.listProjects()
```

## Linting Rules

**Rust** (strict clippy in Cargo.toml):
- `unwrap_used` and `expect_used` are **denied** - use `ok_or()` or `?`
- `pedantic`, `nursery`, `correctness` enabled

**TypeScript/Svelte** (eslint.config.js):
- `@typescript-eslint/no-explicit-any`: error
- `@typescript-eslint/no-floating-promises`: error
- Strict type checking enabled

## Data Flow Example: Running a Job

1. Frontend calls `api.queueJobs(['bench.py'])`
2. Backend inserts job in SQLite (status: pending)
3. Frontend calls `api.startNextJob()`
4. Backend: rsync project → SSH tmux session → update DB (running)
5. Frontend polls `api.getJobStatus()` every 2s
6. Backend: tail logs, parse `[x/y]` progress, detect finish patterns
7. Frontend updates UI with progress, logs, elapsed time

## Remote Execution

- **SSH**: Control socket (ControlMaster) for connection reuse
- **Sync**: rsync for code + pyproject.toml + uv.lock
- **Jobs**: Run in tmux sessions for persistence
- **Logs**: Streamed via `tail` with progress parsing
