# Technology Stack

## Part 1: Frontend (UI)

### Core Framework & Language

| Category         | Technology  | Version | Justification                                                                                           |
| ---------------- | ----------- | ------- | ------------------------------------------------------------------------------------------------------- |
| **UI Framework** | Svelte      | 5.0.0   | Modern reactive framework with runes-based state management, excellent performance, minimal bundle size |
| **Language**     | TypeScript  | 5.6.0   | Type-safe development with strict compiler options enabled                                              |
| **Build Tool**   | Vite        | 7.3.1   | Fast HMR, optimized builds, native ESM support                                                          |
| **Styling**      | TailwindCSS | 4.1.18  | Utility-first CSS framework for rapid UI development                                                    |
| **PostCSS**      | PostCSS     | 8.4.49  | CSS processing with nested syntax support                                                               |

### Tauri Integration

| Category                | Technology                | Version | Justification                       |
| ----------------------- | ------------------------- | ------- | ----------------------------------- |
| **Desktop Integration** | @tauri-apps/api           | 2.x     | IPC communication with Rust backend |
| **Dialogs**             | @tauri-apps/plugin-dialog | 2.x     | Native file/folder picker dialogs   |

### Development Tools

| Category          | Technology   | Version | Justification                                               |
| ----------------- | ------------ | ------- | ----------------------------------------------------------- |
| **Linting**       | ESLint       | 9.39.2  | Code quality enforcement with TypeScript and Svelte plugins |
| **Formatting**    | Prettier     | 3.7.4   | Consistent code formatting with Svelte support              |
| **Type Checking** | svelte-check | 4.3.5   | Svelte component type validation                            |

### Architecture Pattern

**Component-Based Architecture** with Svelte 5 Runes

- Reactive state management using `$state`, `$derived`, and `$effect` runes
- Feature-based organization (benchmarks, jobs, history, SSH)
- Reusable UI component library
- Layout components with resizable panels

### TypeScript Configuration

- **Target**: ES2020
- **Module**: ESNext with bundler resolution
- **Strict Mode**: Enabled
- **Unused Checks**: Enabled for locals and parameters
- **Verbatim Module Syntax**: Enabled for explicit type imports

---

## Part 2: Backend (Core)

### Core Language & Framework

| Category              | Technology | Version      | Justification                                                  |
| --------------------- | ---------- | ------------ | -------------------------------------------------------------- |
| **Language**          | Rust       | Edition 2021 | Memory-safe systems programming, excellent async support       |
| **Desktop Framework** | Tauri      | 2.x          | Secure, lightweight desktop framework with IPC                 |
| **Async Runtime**     | Tokio      | 1.x          | Multi-threaded async runtime with process, sync, time features |

### SSH & Remote Execution

| Category               | Technology  | Version | Justification                                      |
| ---------------------- | ----------- | ------- | -------------------------------------------------- |
| **SSH Client**         | russh       | 0.56    | Pure Rust SSH implementation with aws-lc-rs crypto |
| **SSH Keys**           | russh-keys  | 0.49    | SSH key management and authentication              |
| **Connection Pooling** | bb8         | 0.9     | Async connection pool for SSH sessions             |
| **Async Traits**       | async-trait | 0.1     | Async trait support for connection management      |
| **Security**           | zeroize     | 1.x     | Secure memory wiping for sensitive data            |

### Database

| Category              | Technology | Version      | Justification                                    |
| --------------------- | ---------- | ------------ | ------------------------------------------------ |
| **Database**          | SQLite     | via SQLx 0.8 | Embedded database for projects, benchmarks, jobs |
| **ORM/Query Builder** | SQLx       | 0.8          | Compile-time checked SQL queries, async support  |

### Python Analysis

| Category           | Technology         | Version | Justification                            |
| ------------------ | ------------------ | ------- | ---------------------------------------- |
| **AST Parser**     | tree-sitter        | 0.26    | Fast, incremental parsing of Python code |
| **Python Grammar** | tree-sitter-python | 0.25    | Python language grammar for tree-sitter  |
| **Streaming**      | streaming-iterator | 0.1     | Efficient AST traversal                  |

### Configuration & Utilities

| Category            | Technology                   | Version | Justification                               |
| ------------------- | ---------------------------- | ------- | ------------------------------------------- |
| **Config Format**   | TOML                         | 0.9     | Human-readable configuration files          |
| **Shell Expansion** | shellexpand                  | 3.x     | Expand ~ and environment variables in paths |
| **Date/Time**       | chrono                       | 0.4     | Date and time manipulation                  |
| **Regex**           | regex                        | 1.x     | Pattern matching for log parsing            |
| **Logging**         | tracing + tracing-subscriber | 0.1/0.3 | Structured logging and diagnostics          |

### Serialization

| Category          | Technology | Version | Justification                           |
| ----------------- | ---------- | ------- | --------------------------------------- |
| **Serialization** | serde      | 1.x     | Type-safe serialization/deserialization |
| **JSON**          | serde_json | 1.x     | JSON support for Tauri IPC              |

### Architecture Pattern

**Service-Oriented Architecture with Command Pattern**

- **Commands Layer** (`commands.rs`): 40+ Tauri commands exposing backend functionality
- **State Management** (`state.rs`): Thread-safe shared state with `Arc<Mutex<T>>`
- **Services Layer**:
  - `ssh/` - SSH connection management with connection pooling
  - `db.rs` - Database operations and queries
  - `project.rs` - Python project management via `uv`
  - `python_deps.rs` - Dependency analysis using tree-sitter
  - `job.rs` - Job execution and log parsing
  - `config.rs` - Configuration loading and path resolution

### Build Configuration

**Development Profile**:

- Incremental builds enabled
- No optimization (fast compile)
- Minimal debug symbols
- Dependencies optimized (opt-level 3)

**Release Profile**:

- LTO (Link-Time Optimization) enabled
- Strip debug symbols
- Size optimization (opt-level "z")
- Panic = abort for smaller binary
- Single codegen unit for maximum optimization

### Lint Configuration (SOTA 2026)

**Strict Clippy Rules**:

- `pedantic`, `nursery`, `cargo` groups at warn level
- `correctness` and `suspicious` at deny level
- `unwrap_used` and `expect_used` **denied** (explicit error handling required)
- `unsafe_code` at warn level
- Reasonable exceptions: `module_name_repetitions`, `must_use_candidate`, `too_many_lines`

---

## Integration Architecture

### Tauri IPC (Inter-Process Communication)

- **Frontend → Backend**: TypeScript calls via `@tauri-apps/api` `invoke()`
- **Backend → Frontend**: Rust commands exposed via `#[tauri::command]`
- **Data Format**: JSON serialization via serde
- **Command Count**: 40+ commands for config, SSH, sync, projects, jobs, database

### Communication Flow

```
Frontend (Svelte)
    ↓ invoke("command_name", args)
Tauri IPC Layer (JSON)
    ↓
Backend Command Handler
    ↓ access shared state
AppState (Arc<Mutex<T>>)
    ↓ execute operation
Service Layer (SSH, DB, Projects, Jobs)
    ↓ return result
Response (Result<T, String>)
    ↓ JSON serialization
Frontend receives data
```

### Key Integration Points

1. **Configuration Management**: Frontend reads/writes config via backend commands
2. **SSH Operations**: All SSH connections managed by backend connection pool
3. **Job Management**: Backend queues, starts, monitors jobs; frontend polls for updates
4. **Database Access**: All database operations through backend SQLx layer
5. **File System**: Backend handles rsync, file operations, path resolution

---

## External Dependencies

### Development Environment

- **Node.js Runtime**: Bun (preferred) or Node.js for frontend
- **Rust Toolchain**: Edition 2021 compatible (1.56+)
- **System Dependencies**:
  - SQLite3 development libraries
  - SSH client libraries (for russh)
  - Platform-specific Tauri dependencies

### Remote Execution Requirements

- **SSH Access**: SSH server on remote machines
- **tmux**: For persistent job sessions
- **Python**: For running optimization benchmarks
- **rsync**: For code synchronization

---

## Summary

**Frontend**: Modern Svelte 5 + TypeScript + Vite stack with strict type checking and fast HMR

**Backend**: Rust with Tauri 2, using pure Rust implementations (russh) for security and performance, with async/await throughout

**Integration**: JSON-based IPC with 40+ commands, thread-safe shared state, connection pooling for SSH

**Philosophy**:

- Type safety (TypeScript strict mode, Rust)
- Explicit error handling (no unwrap/expect in Rust)
- Modern tooling (Vite 7, Svelte 5, Tauri 2)
- Security-first (memory safety, zeroize for secrets, strict linting)
- Performance-optimized (LTO, size optimization, connection pooling)
