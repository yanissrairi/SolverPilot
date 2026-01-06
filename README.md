# SolverPilot

A modern GUI application for running optimization solvers and benchmarks on remote servers via SSH.

Built with **Tauri 2** (Rust) + **Svelte 5** + **Tailwind CSS**.

## Features

- **SSH Management**: Automatic key detection from `~/.ssh/config`, passphrase handling via ssh-agent
- **Code Sync**: rsync-based synchronization with dry-run preview
- **Job Queue**: SQLite-backed persistent job queue
- **Remote Execution**: Jobs run in tmux sessions on the remote server
- **Real-time Logs**: Live log streaming with progress parsing (`[x/y]` format)
- **Job Control**: Stop (Ctrl-C) or Kill running jobs
- **History**: View past jobs and their logs

## Screenshots

*Coming soon*

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Bun](https://bun.sh/) or Node.js (18+)
- [mold](https://github.com/rui314/mold) linker (optional, for faster builds on Linux)

### Build from source

```bash
# Clone the repository
git clone https://github.com/yanissrairi/SolverPilot.git
cd SolverPilot

# Install frontend dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

## Configuration

Create a configuration file at `~/.config/solver-pilot/config.toml`:

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

See `config.example.toml` for a complete example.

## Usage

1. **Launch** the application
2. **Enter SSH passphrase** if prompted (stored in ssh-agent for the session)
3. **Select benchmarks** from the list
4. **Sync code** if there are pending changes
5. **Run** selected benchmarks
6. **Monitor** progress and logs in real-time
7. **View history** of past runs

## Development

```bash
# Run with hot-reload
bun run tauri dev

# Type-check frontend
bun run check

# Lint
bun run lint

# Format
bun run format

# Rust checks
cd src-tauri
cargo clippy
cargo fmt
cargo deny check
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust, Tauri 2 |
| Frontend | Svelte 5, TypeScript |
| Styling | Tailwind CSS |
| Database | SQLite (via sqlx) |
| SSH | OpenSSH (ControlMaster) |
| Sync | rsync |

## Project Structure

```
SolverPilot/
├── src/                    # Svelte frontend
│   ├── App.svelte         # Main application
│   └── lib/               # TypeScript utilities
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── commands.rs    # Tauri commands
│   │   ├── ssh.rs         # SSH operations
│   │   ├── db.rs          # SQLite database
│   │   ├── job.rs         # Job parsing
│   │   ├── config.rs      # Configuration
│   │   └── state.rs       # App state
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## License

MIT License - see [LICENSE](LICENSE) file.

## Author

Yanis Srairi
