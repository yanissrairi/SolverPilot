# Development Guide

## Prerequisites

### Required Software

| Tool          | Version                | Purpose                                                   |
| ------------- | ---------------------- | --------------------------------------------------------- |
| **Bun**       | Latest                 | Frontend package manager and runtime (preferred over npm) |
| **Node.js**   | 18+                    | Alternative to Bun if needed                              |
| **Rust**      | Stable (Edition 2021+) | Backend compilation                                       |
| **Tauri CLI** | 2.x                    | Desktop app bundling                                      |

### System Dependencies

**Linux (Ubuntu/Debian)**:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  build-essential \
  curl \
  wget \
  file
```

**macOS**:

```bash
# Xcode Command Line Tools
xcode-select --install
```

**Windows**:

- Visual Studio 2022 with C++ build tools
- WebView2 (usually pre-installed on Windows 10/11)

### Optional Tools

- **cargo-deny**: Security auditing (`cargo install cargo-deny`)
- **lefthook**: Git hooks manager (configured in `lefthook.yml`)

---

## Installation

### 1. Clone Repository

```bash
git clone https://github.com/yanissrairi/SolverPilot.git
cd SolverPilot
```

### 2. Install Frontend Dependencies

```bash
# Using Bun (recommended)
bun install

# Or using npm
npm install
```

### 3. Install Rust Dependencies

Rust dependencies are automatically installed on first build via Cargo.

### 4. Create Configuration File

```bash
# Copy example config
cp config.example.toml config.toml

# Edit config.toml with your settings:
# - SSH host, user, port, key path
# - Remote base directory
# - Gurobi paths
# - uv path
```

---

## Development Commands

### Frontend Development

| Command           | Purpose                            |
| ----------------- | ---------------------------------- |
| `bun run dev`     | Start Vite dev server (hot reload) |
| `bun run build`   | Build production frontend          |
| `bun run preview` | Preview production build           |

### Tauri Development

| Command               | Purpose                          |
| --------------------- | -------------------------------- |
| `bun run tauri dev`   | Run full app in development mode |
| `bun run tauri build` | Build production desktop app     |

### Code Quality

| Command                | Purpose                                |
| ---------------------- | -------------------------------------- |
| `bun run lint`         | Run ESLint with auto-fix               |
| `bun run lint:check`   | Check linting without fixing           |
| `bun run format`       | Format code with Prettier              |
| `bun run format:check` | Check formatting without fixing        |
| `bun run check`        | TypeScript/Svelte type checking        |
| `bun run quality`      | Run all checks (lint + format + types) |

### Rust Commands

Run from `src-tauri/` directory:

| Command                 | Purpose                      |
| ----------------------- | ---------------------------- |
| `cargo fmt`             | Format Rust code             |
| `cargo fmt --check`     | Check formatting             |
| `cargo clippy`          | Run linter with strict rules |
| `cargo check`           | Fast compile check           |
| `cargo build`           | Build debug binary           |
| `cargo build --release` | Build optimized binary       |
| `cargo deny check`      | Security audit               |

---

## Development Workflow

### Standard Development Flow

```bash
# 1. Start development server
bun run tauri dev

# This will:
# - Start Vite dev server on port 5173
# - Compile Rust backend
# - Launch desktop app with hot reload
```

### Frontend-Only Development

```bash
# If you only need to work on UI
bun run dev

# Access at http://localhost:5173
# Note: Backend commands won't work without Tauri
```

### Backend-Only Development

```bash
cd src-tauri
cargo check    # Fast compilation check
cargo clippy   # Linting
cargo build    # Full build
```

---

## Project Structure for Development

### Frontend Hot Reload

Changes to these files trigger instant HMR:

- `src/**/*.svelte` - Svelte components
- `src/**/*.ts` - TypeScript files
- `src/app.css` - Global styles

### Backend Rebuild Required

Changes to these files require app restart:

- `src-tauri/src/**/*.rs` - Rust source files
- `src-tauri/Cargo.toml` - Dependencies

### Configuration

Changes require app restart:

- `config.toml` - User configuration (reloaded on restart)
- `tauri.conf.json` - Tauri configuration

---

## Testing

### Frontend Testing

Currently no automated tests. Manual testing workflow:

1. Start `bun run tauri dev`
2. Test UI interactions
3. Check browser console for errors

### Backend Testing

```bash
cd src-tauri
cargo test
```

### Manual End-to-End Testing

1. Configure `config.toml` with test SSH server
2. Run `bun run tauri dev`
3. Test workflow:
   - Create project
   - Add benchmarks
   - Queue jobs
   - Monitor execution
   - Check history

---

## Build Process

### Development Build

```bash
bun run tauri dev
```

**Output**:

- Frontend: Hot-reloaded via Vite
- Backend: Debug build in `src-tauri/target/debug/`
- Combined: Running desktop app

**Build Time**: ~30s incremental

### Production Build

```bash
bun run tauri build
```

**Output**:

- Platform-specific installer in `src-tauri/target/release/bundle/`
- **Linux**: `.deb`, `.AppImage`
- **macOS**: `.dmg`, `.app`
- **Windows**: `.msi`, `.exe`

**Optimizations**:

- Frontend: Vite production build (minified, tree-shaken)
- Backend: LTO enabled, stripped symbols, size-optimized
- Binary size: ~15-20 MB (depending on platform)

**Build Time**: ~5-10 minutes full build

---

## Git Hooks (lefthook)

Pre-configured hooks in `lefthook.yml`:

### Pre-Commit

Automatically runs on `git commit`:

- Frontend: `bun run lint:check`, `bun run format:check`, `bun run check`
- Backend: `cargo fmt --check`, `cargo clippy`

**Skip hooks** (not recommended):

```bash
git commit --no-verify
```

### Installing Hooks

```bash
# Hooks auto-install on first commit
# Or manually:
lefthook install
```

---

## Common Development Tasks

### Adding a New Frontend Feature

1. Create feature directory:

   ```bash
   mkdir src/lib/features/my-feature
   ```

2. Create component:

   ```typescript
   // src/lib/features/my-feature/MyFeature.svelte
   <script lang="ts">
     interface Props {
       data: string;
     }
     const { data }: Props = $props();
   </script>

   <div>{data}</div>
   ```

3. Use in App.svelte or other component

### Adding a New Backend Command

1. Define function in `src-tauri/src/commands.rs`:

   ```rust
   #[tauri::command]
   async fn my_command(
       state: State<'_, AppState>,
       arg: String,
   ) -> Result<String, String> {
       // Implementation
       Ok(format!("Result: {arg}"))
   }
   ```

2. Register in `src-tauri/src/lib.rs`:

   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       my_command,
   ])
   ```

3. Add TypeScript wrapper in `src/lib/api.ts`:
   ```typescript
   export async function myCommand(arg: string): Promise<string> {
     return invoke('my_command', { arg });
   }
   ```

### Adding a Database Table

1. Update schema in `src-tauri/src/db.rs`:

   ```rust
   sqlx::query(
       r"
       CREATE TABLE IF NOT EXISTS my_table (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           name TEXT NOT NULL
       )
       "
   )
   .execute(&pool)
   .await?;
   ```

2. Add CRUD operations:

   ```rust
   pub async fn insert_my_record(pool: &SqlitePool, name: &str)
       -> Result<i64, String>
   {
       // Implementation
   }
   ```

3. Add to state model in `src-tauri/src/state.rs` if needed

---

## Debugging

### Frontend Debugging

**Chrome DevTools**:

- Right-click in app → "Inspect Element"
- Console, Network, Sources tabs available

**Logging**:

```typescript
console.log('Debug:', data);
console.error('Error:', error);
```

### Backend Debugging

**Logging**:

```rust
use tracing::{info, warn, error, debug};

info!("Starting operation");
debug!("Debug details: {:?}", data);
error!("Error occurred: {}", e);
```

**Environment variables**:

```bash
RUST_LOG=debug bun run tauri dev
RUST_BACKTRACE=1 bun run tauri dev
```

**VS Code Debugging**:

1. Install "CodeLLDB" extension
2. Add launch configuration for Rust debugging

### Database Debugging

**SQLite CLI**:

```bash
sqlite3 solver-pilot.db
.tables
.schema projects
SELECT * FROM jobs WHERE status = 'running';
```

---

## Performance Optimization

### Frontend

- Use `$derived` for computed values (not `$effect`)
- Lazy load features with dynamic imports
- Minimize component re-renders
- Use Skeleton/Spinner for loading states

### Backend

- Connection pooling enabled for SSH (bb8)
- Async/await throughout (Tokio runtime)
- Database queries optimized with indexes
- Incremental log tailing for job monitoring

### Build

Development profile optimizes dependencies:

```toml
[profile.dev.package."*"]
opt-level = 3  # Fast runtime, slow first build
```

---

## Troubleshooting

### "Database not initialized" Error

- Ensure `config.toml` exists and is valid
- Check database path in config
- Verify write permissions

### SSH Connection Fails

- Test SSH manually: `ssh -i ~/.ssh/id_rsa user@host`
- Check SSH key path in `config.toml`
- Verify passphrase if needed
- Ensure remote server is reachable

### Build Errors (Frontend)

```bash
# Clear cache and reinstall
rm -rf node_modules bun.lock
bun install
```

### Build Errors (Backend)

```bash
cd src-tauri
cargo clean
cargo build
```

### Hot Reload Not Working

- Check `vite.config.ts` settings
- Restart dev server
- Clear browser cache

---

## Summary

**Development Environment**:

- Bun + Vite for frontend (fast HMR)
- Rust + Tauri for backend (native performance)
- Lefthook for pre-commit checks

**Key Commands**:

- `bun run tauri dev` - Start development
- `bun run quality` - Check code quality
- `bun run tauri build` - Production build

**Best Practices**:

- Run quality checks before commit
- Test SSH connectivity before job execution
- Use strict TypeScript and Rust linting
- Keep dependencies updated

**Next Steps**:

- See `deployment-guide.md` for production deployment
- See `architecture-patterns.md` for design guidelines
- See `CLAUDE.md` for AI assistant guidance
