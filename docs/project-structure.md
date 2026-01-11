# Project Structure

**Repository Type**: Multi-part Tauri 2 Desktop Application

**Architecture**: Tauri desktop application with separate frontend (Svelte) and backend (Rust) components.

## Parts Overview

### Part 1 - Frontend (UI)

- **Part ID**: `frontend`
- **Location**: `src/`
- **Project Type**: desktop
- **Primary Language**: TypeScript
- **Framework**: Svelte 5
- **Build Tool**: Vite
- **Styling**: TailwindCSS 4
- **Purpose**: User interface for managing and running optimization benchmarks on remote servers

### Part 2 - Backend (Core)

- **Part ID**: `backend`
- **Location**: `src-tauri/`
- **Project Type**: desktop
- **Primary Language**: Rust
- **Framework**: Tauri 2
- **Database**: SQLite (via SQLx)
- **Key Libraries**:
  - russh (SSH client implementation)
  - tree-sitter-python (Python dependency analysis)
  - bb8 (connection pooling)
- **Purpose**: SSH connection management, remote job execution, database operations, Python project analysis

## Integration

Frontend and backend communicate via **Tauri IPC** (Inter-Process Communication):

- Frontend invokes Rust commands via `@tauri-apps/api`
- Backend exposes 40+ Tauri commands for various operations
- Commands handle: config, SSH, sync, projects, jobs, database operations
