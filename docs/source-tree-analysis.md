# Source Tree Analysis

## Project Root Structure

```
SolverPilot/
├── src/                     # Frontend (Svelte 5 + TypeScript)
├── src-tauri/               # Backend (Rust + Tauri 2)
├── docs/                    # Generated documentation (this folder)
├── .github/                 # CI/CD workflows
├── config.toml              # User configuration (SSH, remote paths, Gurobi)
├── solver-pilot.db          # SQLite database (projects, benchmarks, jobs)
├── package.json             # Frontend dependencies
├── CLAUDE.md                # AI assistant guidance
└── README.md                # Project documentation
```

---

## Part 1: Frontend (src/)

### Entry Point

**`src/main.ts`** - Application entry point, mounts root component

### Root Component

**`src/App.svelte`** - Root component with MainLayout and routing logic

### Core API Layer

**`src/lib/api.ts`** - Tauri IPC command wrappers (40+ functions)
**`src/lib/types.ts`** - TypeScript interfaces matching Rust structs

### Directory Structure

```
src/
├── main.ts                  # ✨ Entry point - Mounts App.svelte
├── app.css                  # Global styles with TailwindCSS
├── App.svelte               # 🎯 Root component - MainLayout + state initialization
│
└── lib/
    ├── api.ts               # 🔌 IPC Layer - All Tauri command wrappers
    ├── types.ts             # 📋 Type Definitions - TypeScript interfaces
    │
    ├── features/            # 📦 Feature Modules (domain-driven organization)
    │   ├── benchmarks/
    │   │   └── BenchmarkList.svelte         # Manage benchmark files
    │   ├── jobs/
    │   │   └── JobMonitor.svelte            # Real-time job execution monitoring
    │   ├── history/
    │   │   └── HistoryPanel.svelte          # Job history display
    │   ├── projects/
    │   │   └── ProjectSelector.svelte       # Python project management
    │   ├── dependencies/
    │   │   └── DependencyPanel.svelte       # Dependency analysis & management
    │   ├── ssh/
    │   │   └── SshPassphraseModal.svelte    # SSH key passphrase input
    │   └── setup/
    │       └── SetupWizard.svelte           # First-time configuration wizard
    │
    ├── layout/              # 🏗️ Layout Components
    │   ├── MainLayout.svelte           # 3-panel resizable layout
    │   ├── Header.svelte               # Application header/title bar
    │   └── ResizablePanel.svelte       # Draggable panel divider
    │
    ├── ui/                  # 🎨 Reusable UI Components
    │   ├── Button.svelte               # Primary action button
    │   ├── IconButton.svelte           # Icon-only button
    │   ├── Modal.svelte                # Dialog overlay with focus trap
    │   ├── Select.svelte               # Dropdown selection
    │   ├── Tooltip.svelte              # Hover tooltip
    │   ├── Badge.svelte                # Status indicator
    │   ├── Toast.svelte                # Notification message
    │   ├── ToastContainer.svelte       # Toast manager
    │   ├── Spinner.svelte              # Loading indicator
    │   ├── Skeleton.svelte             # Loading placeholder
    │   └── EmptyState.svelte           # Empty list placeholder
    │
    ├── stores/              # 🗄️ Global State (Svelte 5 Runes)
    │   ├── panels.svelte.ts            # Panel sizes with localStorage
    │   ├── shortcuts.svelte.ts         # Keyboard shortcut registry
    │   └── toast.svelte.ts             # Notification system
    │
    └── utils/               # 🛠️ Utility Functions
        ├── focus-trap.ts               # Modal focus trapping
        └── keyboard.ts                 # Keyboard shortcut matching
```

### Critical Directories

| Directory               | Purpose                    | Key Files                                 |
| ----------------------- | -------------------------- | ----------------------------------------- |
| **`src/lib/features/`** | Domain-specific components | 7 feature modules, 8 main components      |
| **`src/lib/ui/`**       | Reusable UI components     | 11 components (buttons, modals, feedback) |
| **`src/lib/stores/`**   | Global state management    | 3 stores (panels, shortcuts, toasts)      |
| **`src/lib/layout/`**   | Application layout         | 3-panel resizable layout system           |

### Integration Points

**Frontend → Backend**:

- `api.ts` calls → Tauri IPC → `src-tauri/src/commands.rs`
- JSON serialization for all data transfer
- TypeScript types match Rust structs

---

## Part 2: Backend (src-tauri/)

### Entry Points

**`src-tauri/src/main.rs`** - Binary entry point (minimal, calls lib.rs)
**`src-tauri/src/lib.rs`** - Tauri app setup, command registration

### Directory Structure

```
src-tauri/
├── Cargo.toml               # 📦 Rust dependencies & build config
├── tauri.conf.json          # ⚙️ Tauri configuration
├── build.rs                 # 🔧 Build script
├── deny.toml                # 🔒 Cargo-deny security config
├── rustfmt.toml             # 📝 Code formatting rules
│
├── capabilities/            # 🔐 Tauri permissions
│   └── default.json
│
├── icons/                   # 🎨 Application icons
│   ├── icon.png
│   ├── 32x32.png
│   ├── 128x128.png
│   └── 256x256.png
│
└── src/
    ├── main.rs              # ✨ Binary entry point
    ├── lib.rs               # 🎯 Tauri setup - Registers 40+ commands
    │
    ├── state.rs             # 🗄️ AppState - Thread-safe shared state
    ├── commands.rs          # 🔌 Command Layer - All 40+ Tauri commands
    │
    ├── config.rs            # ⚙️ Configuration - Load/save config.toml
    ├── db.rs                # 💾 Database Layer - SQLx operations
    ├── paths.rs             # 📂 Path Utilities - Project/benchmark paths
    │
    ├── project.rs           # 🐍 Project Management - uv integration
    ├── python_deps.rs       # 🔍 Dependency Analysis - Tree-sitter AST
    ├── job.rs               # 📊 Job Management - Log parsing, progress
    │
    └── ssh/                 # 🔐 SSH Module (6 files)
        ├── mod.rs           # Module exports & SshManager
        ├── pool.rs          # Connection pooling (bb8)
        ├── auth.rs          # Authentication (keys, agent)
        ├── executor.rs      # Command execution (tmux, remote jobs)
        ├── transfer.rs      # File transfer (rsync)
        └── error.rs         # Error types
```

### Critical Directories & Files

| File/Module          | Lines | Purpose                                             |
| -------------------- | ----- | --------------------------------------------------- |
| **`lib.rs`**         | ~100  | Tauri app initialization, command registration      |
| **`state.rs`**       | ~110  | Thread-safe app state with `Arc<Mutex<T>>`          |
| **`commands.rs`**    | ~1000 | All 40+ Tauri command implementations               |
| **`db.rs`**          | ~500  | SQLite CRUD operations for projects/benchmarks/jobs |
| **`config.rs`**      | ~200  | TOML config loading/saving, path expansion          |
| **`project.rs`**     | ~400  | Python project management via `uv`                  |
| **`python_deps.rs`** | ~800  | Tree-sitter Python AST analysis for imports         |
| **`job.rs`**         | ~120  | Log parsing, progress extraction `[x/y]`            |
| **`ssh/`**           | ~600  | SSH module with connection pooling                  |

### SSH Module Details

```
ssh/
├── mod.rs           # SshManager - High-level SSH operations
├── pool.rs          # Connection pooling with bb8
├── auth.rs          # SSH key loading & authentication
├── executor.rs      # Remote command execution via tmux
├── transfer.rs      # rsync-based file transfer
└── error.rs         # SSH error types
```

**Key Features**:

- **Connection Pooling**: bb8 pool for SSH connection reuse
- **Authentication**: SSH key with optional passphrase, agent support
- **Execution**: tmux-based persistent sessions for jobs
- **Transfer**: rsync for efficient code synchronization
- **Error Handling**: Custom error types with context

### Service Layer Architecture

```
┌─────────────────────────────────────────┐
│      Commands Layer (commands.rs)       │  ← 40+ Tauri commands
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│     Shared State (state.rs)             │  ← Arc<Mutex<T>> wrappers
│  - Config                                │
│  - Database Pool                         │
│  - SSH Manager                           │
│  - Current Job/Project                   │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│       Service Modules                   │
├─────────────────────────────────────────┤
│  config.rs    - Configuration           │
│  db.rs        - Database operations     │
│  ssh/         - SSH management          │
│  project.rs   - Python projects (uv)    │
│  python_deps.rs - Dependency analysis   │
│  job.rs       - Job log parsing         │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│     Infrastructure                      │
│  - SQLite (via SQLx)                    │
│  - SSH (via russh + bb8)                │
│  - Filesystem (tokio fs)                │
│  - Remote Server (tmux, rsync)          │
└─────────────────────────────────────────┘
```

---

## Integration Between Parts

### IPC Communication Flow

```
Frontend (Svelte)
    │
    │ api.ts wrapper functions
    ↓
@tauri-apps/api
    │
    │ invoke('command_name', { args })
    ↓
Tauri IPC Layer
    │
    │ JSON serialization
    ↓
Backend (Rust)
    │
    │ commands.rs - Command handler
    ↓
AppState Access
    │
    │ Lock mutexes
    ↓
Service Layer
    │
    │ Business logic execution
    ↓
Result<T, String>
    │
    │ JSON serialization
    ↓
Frontend receives response
```

### Data Flow Example: Starting a Job

```
1. Frontend: startNextJob()
   ↓
2. api.ts: invoke('start_next_job')
   ↓
3. Tauri IPC: JSON → Rust
   ↓
4. commands.rs: start_next_job(state)
   ↓
5. state.rs: Lock db, ssh_manager
   ↓
6. db.rs: Find first pending job
   ↓
7. ssh/transfer.rs: rsync project files
   ↓
8. ssh/executor.rs: Start tmux session
   ↓
9. db.rs: Update job status to 'running'
   ↓
10. Result → JSON → Frontend
```

---

## Configuration & Data Files

### User Configuration

**`config.toml`** - User configuration (not in git)

- SSH connection details (host, user, port, key_path)
- Remote base directory
- Gurobi settings (GUROBI_HOME, license file)
- Tools paths (uv)
- Polling interval

### Database

**`solver-pilot.db`** - SQLite database

- **projects** table - Python project environments
- **benchmarks** table - Benchmark file references
- **jobs** table - Job execution history and status

### Project Data

**`projects/{name}/`** - Per-project directories

- `.python-version` - Python version file
- `pyproject.toml` - uv project manifest
- `uv.lock` - Dependency lockfile
- `.venv/` - Virtual environment
- `benchmarks/` - Benchmark Python files

---

## Build & Development Files

### Frontend Build

- **`package.json`** - Node dependencies, scripts
- **`vite.config.ts`** - Vite build configuration
- **`tsconfig.json`** - TypeScript compiler options
- **`svelte.config.js`** - Svelte preprocessor config
- **`eslint.config.js`** - ESLint rules
- **`.prettierrc`** - Prettier formatting rules
- **`postcss.config.js`** - PostCSS (TailwindCSS) config

### Backend Build

- **`Cargo.toml`** - Rust dependencies, features, lints
- **`build.rs`** - Tauri build script
- **`rustfmt.toml`** - Rust formatting rules
- **`deny.toml`** - Cargo-deny security checks

### CI/CD

**`.github/workflows/`**:

- `ci.yml` - Continuous integration (lint, test, build)
- `release.yml` - Release automation
- `claude-code-review.yml` - Automated code review
- `claude.yml` - Claude-specific workflow

---

## Summary

**Total Files**: ~100 source files (excluding dependencies)

**Frontend**:

- **Entry**: `src/main.ts` → `App.svelte`
- **Features**: 7 modules, 8 main components
- **UI Library**: 11 reusable components
- **State**: 3 global stores
- **API Layer**: 40+ typed wrappers

**Backend**:

- **Entry**: `src-tauri/src/main.rs` → `lib.rs`
- **Commands**: 40+ Tauri commands in `commands.rs`
- **Services**: 6 core modules + SSH module (6 files)
- **Database**: SQLx with 3 tables
- **Architecture**: Service-oriented with command pattern

**Integration**:

- Tauri IPC with JSON serialization
- Type-safe interfaces (TypeScript ↔ Rust)
- Connection pooling for SSH
- Real-time polling for job status
