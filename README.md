# SolverPilot

![Alpha](https://img.shields.io/badge/status-alpha-orange)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A modern desktop application for running Python optimization benchmarks on remote servers via SSH.

Built with **Tauri 2** (Rust) + **Svelte 5** + **Tailwind CSS**.

---

## Why SolverPilot?

If you're a researcher running optimization benchmarks on remote servers, you know the pain:

- **🌀 Terminal chaos**: Multiple SSH sessions, scattered tmux windows, lost context
- **📊 No visibility**: Can't see what's running, no progress tracking, hard to find logs
- **🔄 Manual sync hell**: Forgetting to sync code, version mismatches, tedious rsync commands
- **💥 Fragile workflows**: Sessions die on disconnect, no history, hard to resume

**SolverPilot solves this** with a unified GUI that brings order to the chaos:

- ✅ Single interface for all your remote benchmarks
- ✅ **Persistent jobs** that survive disconnects (SQLite + tmux)
- ✅ Real-time progress tracking and log streaming
- ✅ Smart code sync with rsync
- ✅ Complete job history and searchable logs

### vs. Manual SSH + tmux

| Aspect                       | Manual SSH + tmux             | SolverPilot                |
| ---------------------------- | ----------------------------- | -------------------------- |
| **Multi-session management** | Multiple terminal windows     | Single unified interface   |
| **Job persistence**          | Manual tmux attach            | Automatic reconnection     |
| **Progress tracking**        | grep logs manually            | Real-time `[x/y]` parsing  |
| **History**                  | Lost when session ends        | SQLite database, permanent |
| **Code sync**                | rsync by hand, easy to forget | Integrated rsync in GUI    |
| **Learning curve**           | Need tmux/SSH expertise       | GUI for common workflows   |

**SolverPilot doesn't replace SSH** - it makes it better for the specific use case of running and monitoring Python benchmarks remotely.

---

## Features

### Core Capabilities

- **🔐 SSH Management**: Auto-detect keys from `~/.ssh/config`, passphrase via ssh-agent, persistent connections
- **📦 Smart Code Sync**: rsync-based synchronization, Python dependency analysis (tree-sitter AST)
- **📋 Job Queue**: SQLite-backed persistent queue, survives app restarts
- **🎯 Remote Execution**: Jobs run in tmux sessions on the remote server
- **📊 Real-time Monitoring**: Live log streaming with progress parsing (`[x/y]` format detection)
- **⚙️ Job Control**: Stop (Ctrl-C) or Kill running jobs
- **📜 Complete History**: Searchable logs, job metadata, execution times

### Advanced Features

- **🧙 Setup Wizard**: Guided 4-step onboarding with SSH connection testing
- **3-Panel Resizable Layout**: Code/benchmarks, queue, and logs visible simultaneously
- **Python Project Management**: Automatic `uv` integration for dependencies
- **Keyboard Shortcuts**: Navigate without touching the mouse
- **Toast Notifications**: Non-intrusive status updates
- **Focus Management**: Smart focus trapping for modals and panels

---

## Installation

### Prerequisites

- **[Rust](https://rustup.rs/)** (1.70+)
- **[Bun](https://bun.sh/)** or Node.js (18+)
- **OpenSSH** client
- **rsync** (typically pre-installed on Linux/macOS)

> **Platform Support**: Linux and macOS. Windows is not currently supported due to dependencies on rsync and OpenSSH CLI tools (not natively available on Windows). May work via WSL2 but remains untested.

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yanissrairi/SolverPilot.git
cd SolverPilot

# Install frontend dependencies
bun install

# Run in development mode (hot-reload)
bun run tauri dev

# Build for production
bun run tauri build
```

The production binary will be in `src-tauri/target/release/`.

---

## Configuration

### First Launch: Setup Wizard

On first launch, a **4-step setup wizard** guides you through configuration:

1. **SSH Connection**: Host, user, port, SSH key path
2. **Authentication**: Test connection with optional passphrase
3. **Paths**: Remote working directory, `uv` path
4. **Optional**: Gurobi environment variables (if needed)

The wizard creates `~/.config/solver-pilot/config.toml` automatically.

### Manual Configuration

You can also create or edit the configuration file at `~/.config/solver-pilot/config.toml`:

```toml
[ssh]
host = "your-server"          # SSH host (from ~/.ssh/config or IP)
user = "username"             # SSH username
use_agent = true              # Use ssh-agent for authentication

[paths]
local_code = "/path/to/your/project"           # Local project root
local_benchmarks = "/path/to/benchmarks"       # Directory containing benchmark_*.py files
local_results = "/path/to/results"             # Where to store results
remote_base = "~/benchmarks"                   # Remote working directory

[polling]
interval_seconds = 2          # Log refresh interval
```

See [`config.example.toml`](config.example.toml) for all available options.

---

## Usage

### First Time

1. **Launch** the application
2. **Complete the setup wizard** (SSH connection, paths, optional Gurobi)
3. **Test connection** - the wizard validates your SSH configuration

### Daily Workflow

1. **Launch** the application (wizard runs only on first launch)
2. **Enter SSH passphrase** if prompted (stored in ssh-agent for the session)
3. **Select benchmarks** from the list (Python files matching `benchmark_*.py`)
4. **Sync code** if there are pending changes
5. **Queue jobs** and start execution
6. **Monitor** progress and logs in real-time
7. **View history** of past runs with full logs

### Tips

- Jobs continue running even if you close the app (tmux persistence)
- Use keyboard shortcuts for faster navigation
- Sync status shows which files need updating
- Logs are parsed automatically for `[x/y]` progress patterns

---

## Development

```bash
# Development with hot-reload
bun run tauri dev

# Frontend type-check
bun run check

# Lint & format
bun run lint          # ESLint with auto-fix
bun run format        # Prettier
bun run quality       # Run all checks (lint + format + type-check)

# Rust checks
cd src-tauri
cargo clippy          # Strict linting (see Cargo.toml)
cargo fmt             # Format Rust code
cargo deny check      # Check dependencies for security issues
```

### Code Quality Standards

- **Rust**: Strict Clippy rules enforced (no `unwrap`, no `expect`, pedantic mode)
- **TypeScript**: No `any`, no floating promises, strict type checking
- All errors returned as `Result<T, String>` in Rust commands
- Svelte 5 runes (no legacy stores)

---

## Tech Stack

| Component    | Technology               |
| ------------ | ------------------------ |
| **Backend**  | Rust, Tauri 2            |
| **Frontend** | Svelte 5, TypeScript     |
| **Styling**  | Tailwind CSS             |
| **Database** | SQLite (via sqlx)        |
| **SSH**      | OpenSSH client           |
| **Sync**     | rsync                    |
| **Parser**   | tree-sitter (Python AST) |
| **Build**    | Bun, Vite, Cargo         |

---

## Project Structure

```
SolverPilot/
├── src/                           # Svelte 5 frontend
│   ├── App.svelte                 # Main application
│   ├── lib/
│   │   ├── features/              # Domain components (benchmarks, jobs, history, ssh)
│   │   ├── layout/                # MainLayout, Header, ResizablePanel
│   │   ├── stores/                # Svelte 5 runes stores (panels, shortcuts, toast)
│   │   ├── ui/                    # Reusable components (Button, Modal, Badge, Toast)
│   │   ├── utils/                 # Utilities (focus-trap, keyboard)
│   │   ├── api.ts                 # Tauri invoke wrappers
│   │   └── types.ts               # TypeScript interfaces
│   └── main.ts
├── src-tauri/                     # Rust backend
│   ├── src/
│   │   ├── lib.rs                 # Tauri setup, 40+ commands registered
│   │   ├── commands.rs            # All Tauri commands (config, ssh, sync, projects, jobs)
│   │   ├── state.rs               # Thread-safe AppState with Arc<Mutex<T>>
│   │   ├── config.rs              # Load config.toml, path helpers
│   │   ├── db.rs                  # SQLite via sqlx (projects, benchmarks, jobs tables)
│   │   ├── ssh.rs                 # SSH control socket, rsync, tmux job management
│   │   ├── project.rs             # Python project management via uv
│   │   ├── python_deps.rs         # Tree-sitter Python AST analysis for imports
│   │   └── job.rs                 # Log parsing, progress extraction [x/y]
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

---

## Known Limitations

This project is in **alpha** and primarily for personal use. Here's what to expect:

- ⚠️ **Python only**: Other languages not yet supported
- ⚠️ **Limited test coverage**: No automated tests yet, alpha quality
- ⚠️ **Breaking changes possible**: API and config format may change without notice

Use at your own risk. Backup your data.

---

## Contributing

**Issues welcome!** If you find bugs or have feature suggestions, please [open an issue](https://github.com/yanissrairi/SolverPilot/issues).

**Pull requests**: Not accepting PRs at this time while the project is in alpha and the architecture is stabilizing. This may change in the future.

If you want to fork and experiment, feel free! The codebase is MIT licensed.

---

## License

MIT License - see [LICENSE](LICENSE) file.

You are free to use, modify, and distribute this software. See the license file for full terms.

---

## Author

**Yanis Srairi** - [GitHub](https://github.com/yanissrairi)

---

## Acknowledgments

Built with excellent open-source tools:

- [Tauri](https://tauri.app/) - Rust-powered desktop framework
- [Svelte](https://svelte.dev/) - Reactive UI framework
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [tree-sitter](https://tree-sitter.github.io/) - Incremental parsing system

---

**Status**: Alpha - Active development, personal use, breaking changes expected.
