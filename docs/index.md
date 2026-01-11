# SolverPilot - Project Documentation Index

**Generated**: 2026-01-07
**Project Type**: Multi-part Tauri 2 Desktop Application
**Scan Level**: Exhaustive

---

## 📋 Project Overview

**SolverPilot** is a GUI desktop application for running Python optimization benchmarks on remote servers via SSH.

### Quick Reference

| Aspect                 | Details                                        |
| ---------------------- | ---------------------------------------------- |
| **Repository Type**    | Multi-part (Frontend + Backend)                |
| **Architecture**       | Tauri 2 desktop app with IPC communication     |
| **Primary Languages**  | TypeScript (Frontend), Rust (Backend)          |
| **Frontend Framework** | Svelte 5 with runes-based state management     |
| **Backend Framework**  | Tauri 2 with Tokio async runtime               |
| **Database**           | SQLite via SQLx                                |
| **Integration**        | 40+ Tauri IPC commands with JSON serialization |

### Project Parts

#### Part 1: Frontend (UI)

- **Location**: `src/`
- **Type**: Desktop UI
- **Tech Stack**: Svelte 5, TypeScript, Vite 7, TailwindCSS 4
- **Entry Point**: `src/main.ts` → `App.svelte`
- **Components**: 28+ (7 feature modules, 11 UI components)

#### Part 2: Backend (Core)

- **Location**: `src-tauri/`
- **Type**: Desktop backend
- **Tech Stack**: Rust (Edition 2021), Tauri 2, russh (SSH), SQLx (SQLite)
- **Entry Point**: `src-tauri/src/main.rs` → `lib.rs`
- **Modules**: 6 core services + SSH module (6 files)

---

## 📚 Generated Documentation

### Core Architecture

| Document                                                      | Description                                                                   |
| ------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| **[Project Structure](./project-structure.md)**               | Repository organization and parts overview                                    |
| **[Architecture Patterns](./architecture-patterns.md)**       | Design patterns for frontend (component-based) and backend (service-oriented) |
| **[Integration Architecture](./integration-architecture.md)** | How frontend and backend communicate via Tauri IPC                            |
| **[Source Tree Analysis](./source-tree-analysis.md)**         | Complete annotated directory structure (~100 source files)                    |

### Technology & Stack

| Document                                          | Description                                                         |
| ------------------------------------------------- | ------------------------------------------------------------------- |
| **[Technology Stack](./technology-stack.md)**     | Complete tech stack for both parts with versions and justifications |
| **[IPC Commands](./ipc-commands-integration.md)** | All 40+ Tauri commands documented with examples                     |
| **[Data Models](./data-models-backend.md)**       | Database schema (3 tables) and Rust data models                     |

### Frontend Documentation

| Document                                                           | Description                                             |
| ------------------------------------------------------------------ | ------------------------------------------------------- |
| **[UI Component Inventory](./ui-component-inventory-frontend.md)** | All 28+ components categorized by type                  |
| **[State Management](./state-management-patterns-frontend.md)**    | Svelte 5 runes-based state management (3 global stores) |

### Development & Deployment

| Document                                        | Description                                            |
| ----------------------------------------------- | ------------------------------------------------------ |
| **[Development Guide](./development-guide.md)** | Setup, installation, dev commands, workflow, debugging |
| **[Deployment Guide](./deployment-guide.md)**   | CI/CD pipeline, multi-platform builds, release process |

### Project Metadata

| Document                                                            | Description                                                      |
| ------------------------------------------------------------------- | ---------------------------------------------------------------- |
| **[Existing Documentation](./existing-documentation-inventory.md)** | Inventory of pre-existing docs (README, CLAUDE.md, CI workflows) |
| **[Project Parts Metadata](./project-parts-metadata.json)**         | Machine-readable project structure metadata                      |

---

## 🎯 Quick Start for AI Agents

### Understanding the Codebase

**For UI/Frontend Tasks**:

1. Read [UI Component Inventory](./ui-component-inventory-frontend.md) for component overview
2. Check [State Management](./state-management-patterns-frontend.md) for reactive patterns
3. Review [Architecture Patterns](./architecture-patterns.md) for Svelte 5 runes usage
4. Consult [IPC Commands](./ipc-commands-integration.md) for backend communication

**For Backend/API Tasks**:

1. Read [Data Models](./data-models-backend.md) for database schema
2. Check [IPC Commands](./ipc-commands-integration.md) for command implementations
3. Review [Architecture Patterns](./architecture-patterns.md) for service layer design
4. Consult [Source Tree Analysis](./source-tree-analysis.md) for module locations

**For Full-Stack Features**:

1. Start with [Integration Architecture](./integration-architecture.md) for data flow
2. Check [IPC Commands](./ipc-commands-integration.md) for command patterns
3. Review [Technology Stack](./technology-stack.md) for framework capabilities
4. Consult both frontend and backend docs as needed

### Development Setup

1. **Prerequisites**: Bun, Rust, Tauri CLI → [Development Guide](./development-guide.md#prerequisites)
2. **Installation**: Clone, install deps → [Development Guide](./development-guide.md#installation)
3. **Running**: `bun run tauri dev` → [Development Guide](./development-guide.md#development-commands)

### Common Tasks

| Task                       | Primary Documents                                                                                                                  |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| **Add new UI component**   | [UI Component Inventory](./ui-component-inventory-frontend.md), [Architecture Patterns](./architecture-patterns.md)                |
| **Add new Tauri command**  | [IPC Commands](./ipc-commands-integration.md), [Development Guide](./development-guide.md#adding-a-new-backend-command)            |
| **Modify database schema** | [Data Models](./data-models-backend.md), [Development Guide](./development-guide.md#adding-a-database-table)                       |
| **Add feature module**     | [Source Tree Analysis](./source-tree-analysis.md), [Architecture Patterns](./architecture-patterns.md)                             |
| **Debug SSH issues**       | [Integration Architecture](./integration-architecture.md#4-ssh-integration), [Development Guide](./development-guide.md#debugging) |
| **Build for production**   | [Deployment Guide](./deployment-guide.md#build-process)                                                                            |

---

## 🗂️ Project File Organization

### Frontend (src/)

```
src/
├── main.ts                  # Entry point
├── App.svelte               # Root component
└── lib/
    ├── api.ts               # IPC command wrappers (40+ functions)
    ├── types.ts             # TypeScript interfaces
    ├── features/            # 7 feature modules
    │   ├── benchmarks/
    │   ├── jobs/
    │   ├── history/
    │   ├── projects/
    │   ├── dependencies/
    │   ├── ssh/
    │   └── setup/
    ├── ui/                  # 11 reusable components
    ├── layout/              # 3-panel layout system
    ├── stores/              # 3 global stores (panels, shortcuts, toast)
    └── utils/               # Utilities (focus-trap, keyboard)
```

### Backend (src-tauri/)

```
src-tauri/src/
├── main.rs                  # Binary entry point
├── lib.rs                   # Tauri setup, command registration
├── commands.rs              # 40+ Tauri command implementations
├── state.rs                 # Thread-safe AppState
├── config.rs                # Configuration management
├── db.rs                    # SQLite operations (3 tables)
├── project.rs               # Python project management (uv)
├── python_deps.rs           # Tree-sitter AST analysis
├── job.rs                   # Log parsing, progress tracking
├── paths.rs                 # Path utilities
└── ssh/                     # SSH module (6 files)
    ├── mod.rs               # SshManager
    ├── pool.rs              # Connection pooling (bb8)
    ├── auth.rs              # Authentication
    ├── executor.rs          # Command execution (tmux)
    ├── transfer.rs          # File transfer (rsync)
    └── error.rs             # Error types
```

---

## 🔗 External Resources

### Existing Project Documentation

| Document         | Location              | Purpose                                                |
| ---------------- | --------------------- | ------------------------------------------------------ |
| **README.md**    | `/README.md`          | Project overview, features, quick start                |
| **CLAUDE.md**    | `/CLAUDE.md`          | AI assistant guidance, architecture overview, patterns |
| **CI Workflows** | `/.github/workflows/` | Automated builds, tests, releases                      |

### Related Technologies

- **[Svelte 5 Docs](https://svelte.dev/)** - Frontend framework (runes, components)
- **[Tauri Docs](https://tauri.app/)** - Desktop framework (IPC, build, packaging)
- **[Rust Docs](https://doc.rust-lang.org/)** - Backend language
- **[SQLx Docs](https://docs.rs/sqlx/)** - Database library
- **[russh Docs](https://docs.rs/russh/)** - SSH client library

---

## 📊 Project Statistics

| Metric                      | Value                                 |
| --------------------------- | ------------------------------------- |
| **Total Source Files**      | ~100 (excluding dependencies)         |
| **Frontend Components**     | 28+                                   |
| **Backend Modules**         | 11 (6 core + SSH module with 6 files) |
| **Tauri Commands**          | 40+                                   |
| **Database Tables**         | 3 (projects, benchmarks, jobs)        |
| **Global Stores**           | 3 (panels, shortcuts, toast)          |
| **Lines of Code (Backend)** | ~3,240 (src-tauri/src/)               |

---

## 🏗️ Architecture Summary

### Design Principles

1. **Type Safety**: TypeScript strict mode + Rust's type system
2. **Explicit Error Handling**: No `unwrap`/`expect` in Rust (clippy enforced)
3. **Separation of Concerns**: Clear frontend/backend boundaries
4. **Performance**: Connection pooling, async I/O, incremental updates
5. **Security**: Memory safety, credential protection, input validation
6. **Maintainability**: Clear structure, consistent patterns, comprehensive linting

### Key Patterns

- **Frontend**: Component-based with Svelte 5 runes, feature-driven organization
- **Backend**: Service-oriented with command pattern, Arc<Mutex<T>> state
- **Integration**: Tauri IPC with JSON, 40+ typed commands
- **Data Flow**: Polling for real-time, request-response for actions
- **Error Handling**: Result<T, String> propagation, toast notifications

---

## 🚀 Next Steps for Development

### For New Features

1. **Read relevant architecture docs** to understand existing patterns
2. **Check component/module inventory** to avoid duplication
3. **Follow established patterns** documented in Architecture Patterns
4. **Add types** to both TypeScript (types.ts) and Rust (state.rs)
5. **Implement IPC commands** following the command pattern
6. **Test locally** with `bun run tauri dev`
7. **Run quality checks** with `bun run quality` and `cargo clippy`

### For Bug Fixes

1. **Identify the layer** (UI, IPC, service, database, SSH)
2. **Consult relevant docs** (see Quick Start section above)
3. **Check integration points** in Integration Architecture
4. **Debug with appropriate tools** (DevTools for frontend, tracing for backend)
5. **Test the fix** end-to-end
6. **Verify with quality checks**

### For Maintenance

1. **Update dependencies** regularly (Dependabot PRs, `cargo update`)
2. **Run security audits** (`cargo deny check`)
3. **Monitor CI/CD** builds (GitHub Actions)
4. **Review and update docs** as code evolves
5. **Check for deprecations** in Svelte 5, Tauri 2, Rust

---

## 📝 Documentation Metadata

**Workflow Version**: 1.2.0
**Scan Mode**: initial_scan
**Scan Level**: exhaustive
**Generated Files**: 15 markdown documents + 2 JSON metadata files
**Total Documentation Size**: Comprehensive coverage of all project aspects

**Generated by**: BMM document-project workflow
**Date**: 2026-01-07

---

## 💡 Tips for AI-Assisted Development

### Best Practices

1. **Always start here** (index.md) to get oriented
2. **Read architecture docs** before making cross-cutting changes
3. **Follow existing patterns** documented in Architecture Patterns
4. **Check IPC Commands doc** before adding new backend functionality
5. **Consult State Management** for frontend reactive patterns
6. **Reference CLAUDE.md** for additional code guidelines

### Common Pitfalls to Avoid

1. ❌ Don't use `unwrap()` or `expect()` in Rust (clippy will deny)
2. ❌ Don't access database directly from frontend (use IPC commands)
3. ❌ Don't use legacy Svelte stores (use $state, $derived, $effect runes)
4. ❌ Don't skip type annotations (TypeScript strict mode enabled)
5. ❌ Don't bypass error handling (always return Result<T, String>)

### When in Doubt

- **Frontend questions** → UI Component Inventory, State Management
- **Backend questions** → Data Models, IPC Commands, Source Tree
- **Integration questions** → Integration Architecture
- **Build/deploy questions** → Development Guide, Deployment Guide
- **General architecture** → Architecture Patterns, Technology Stack

---

**This index is your starting point for all development work on SolverPilot. Bookmark it!**
