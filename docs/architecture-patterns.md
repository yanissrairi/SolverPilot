# Architecture Patterns

## Frontend Architecture Pattern

### Pattern: Component-Based Architecture with Svelte 5 Runes

**Description**: The frontend follows a modern component-based architecture using Svelte 5's new runes system for reactive state management.

**Key Characteristics**:

1. **Feature-Based Organization**: Code organized by feature domains (benchmarks, jobs, history, SSH)
2. **Runes-Based Reactivity**: Uses `$state`, `$derived`, and `$effect` runes instead of legacy stores
3. **Component Composition**: Reusable UI components with snippet-based children
4. **Layout Components**: Resizable 3-panel layout with main content areas

**Directory Structure Pattern**:

```
src/
├── lib/
│   ├── features/        # Feature-specific components
│   │   ├── benchmarks/  # Benchmark management
│   │   ├── jobs/        # Job execution and monitoring
│   │   ├── history/     # Job history
│   │   └── ssh/         # SSH connection management
│   ├── layout/          # Layout components
│   │   ├── MainLayout   # 3-panel layout system
│   │   ├── Header       # Application header
│   │   └── ResizablePanel  # Resizable panel component
│   ├── stores/          # Svelte 5 runes stores
│   ├── ui/              # Reusable UI components
│   ├── utils/           # Utilities (focus-trap, keyboard)
│   ├── api.ts           # Tauri invoke wrappers
│   └── types.ts         # TypeScript interfaces
├── App.svelte           # Root component
└── main.ts              # Application entry point
```

**State Management Strategy**:

- **Local Component State**: `$state` runes for component-specific data
- **Derived Values**: `$derived` for computed values
- **Side Effects**: `$effect` for reactions to state changes
- **Global State**: Minimal, stored in runes-based stores

**API Communication Pattern**:

- Centralized API layer (`lib/api.ts`) wrapping Tauri invoke calls
- Type-safe interfaces for all backend commands
- Error handling at the API boundary

---

## Backend Architecture Pattern

### Pattern: Service-Oriented Architecture with Command Pattern

**Description**: The backend implements a service-oriented architecture where business logic is organized into services, exposed to the frontend through Tauri commands.

**Key Characteristics**:

1. **Command Pattern**: All frontend-facing operations exposed as Tauri commands
2. **Service Layer**: Business logic encapsulated in service modules
3. **Shared State**: Thread-safe state management using `Arc<Mutex<T>>`
4. **Connection Pooling**: Reusable SSH connections via bb8 pool
5. **Repository Pattern**: Database access abstracted through SQLx

**Module Structure**:

```
src-tauri/src/
├── lib.rs              # Tauri setup, registers 40+ commands
├── main.rs             # Binary entry point
├── state.rs            # AppState with Arc<Mutex<T>>
├── commands.rs         # All Tauri command handlers
├── config.rs           # Configuration loading
├── db.rs               # Database operations (SQLx)
├── ssh/                # SSH module
│   ├── mod.rs          # SSH manager with connection pooling
│   ├── connection.rs   # SSH connection implementation
│   └── pool.rs         # bb8 connection pool
├── project.rs          # Python project management (uv)
├── python_deps.rs      # Tree-sitter AST analysis
├── job.rs              # Job log parsing, progress tracking
└── paths.rs            # Path utilities
```

**Layered Architecture**:

```
┌─────────────────────────────────────┐
│      Tauri Commands Layer          │  ← 40+ commands in commands.rs
│   (Frontend-facing API boundary)   │
└─────────────────┬───────────────────┘
                  ↓
┌─────────────────────────────────────┐
│       Shared State Layer            │  ← Arc<Mutex<AppState>>
│  (Config, DB, SSH Pool, Job Queue)  │
└─────────────────┬───────────────────┘
                  ↓
┌─────────────────────────────────────┐
│        Service Layer                │  ← Business logic modules
│  (SSH, DB, Project, Job, Config)    │
└─────────────────┬───────────────────┘
                  ↓
┌─────────────────────────────────────┐
│      Infrastructure Layer           │  ← External systems
│  (SQLite, SSH, Filesystem, Remote)  │
└─────────────────────────────────────┘
```

**State Management**:

```rust
pub struct AppState {
    pub config: Arc<Mutex<Option<Config>>>,
    pub db: Arc<Mutex<Option<SqlitePool>>>,
    pub ssh_pool: Arc<Mutex<Option<SshConnectionPool>>>,
    pub job_queue: Arc<Mutex<JobQueue>>,
}
```

**Error Handling Pattern**:

- All commands return `Result<T, String>`
- No `unwrap()` or `expect()` (enforced by clippy deny rules)
- Explicit error propagation using `?` operator
- User-friendly error messages for frontend

**Concurrency Pattern**:

- Tokio runtime with multi-threading
- `Arc<Mutex<T>>` for shared mutable state
- Connection pooling for SSH reuse
- Async/await throughout

---

## Cross-Cutting Patterns

### IPC Communication Pattern

**Request-Response with JSON Serialization**

- Frontend invokes command by name with typed arguments
- Backend deserializes, processes, serializes response
- Type safety enforced by serde and TypeScript interfaces

### Data Flow Pattern

**Unidirectional with Polling**

1. Frontend initiates action (e.g., start job)
2. Backend queues operation, returns immediately
3. Backend processes asynchronously
4. Frontend polls for status updates
5. Backend streams progress via log parsing

### Security Patterns

1. **Credential Management**: SSH keys loaded from config, never exposed to frontend
2. **Memory Safety**: Rust's ownership system prevents memory bugs
3. **Sensitive Data**: `zeroize` crate for secure memory wiping
4. **Input Validation**: All external inputs validated at command boundary

### Async Patterns

**Backend**:

- Tokio async runtime
- Async SSH operations
- Async database queries
- Async file I/O

**Frontend**:

- Promise-based API calls
- Async component loading
- Reactive updates via runes

---

## Pattern Justifications

### Why Component-Based (Frontend)?

- Svelte 5 provides excellent reactivity with minimal boilerplate
- Runes offer better TypeScript integration than legacy stores
- Component composition enables reusable UI patterns
- Feature-based organization scales with project growth

### Why Service-Oriented (Backend)?

- Clear separation of concerns
- Testable business logic
- Easier to maintain and extend
- Connection pooling improves performance
- Command pattern provides clean frontend API

### Why Multi-Part Architecture?

- Frontend and backend have different concerns and languages
- Tauri provides secure, lightweight bridge
- Rust backend offers performance and safety
- Svelte frontend offers productivity and bundle size
- Clear separation enables independent testing and deployment

---

## Design Principles

1. **Type Safety**: TypeScript + Rust eliminate entire classes of bugs
2. **Explicit Error Handling**: No silent failures, all errors propagated
3. **Separation of Concerns**: Clear boundaries between layers
4. **Performance**: Connection pooling, async I/O, optimized builds
5. **Security**: Memory safety, credential protection, strict linting
6. **Maintainability**: Clear structure, consistent patterns, comprehensive linting
