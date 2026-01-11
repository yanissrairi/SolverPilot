---
project_name: 'SolverPilot'
user_name: 'Yanis'
date: '2026-01-08'
sections_completed:
  [
    'technology_stack',
    'language_specific',
    'framework_specific',
    'testing',
    'code_quality',
    'workflow',
    'critical_rules',
  ]
status: 'complete'
existing_patterns_found: 22
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. Focus on unobvious details that agents might otherwise miss._

---

## Technology Stack & Versions

**Frontend:**

- Svelte **5.0.0** - Runes-based ($state, $derived, $effect) - NO legacy stores
- TypeScript **5.6.0** - Strict mode (noUnusedLocals, noUnusedParameters, noFallthroughCasesInSwitch)
- Vite **7.3.1** - ES2020 target
- TailwindCSS **4.1.18**

**Backend:**

- Rust **Edition 2021** - Strict clippy (unwrap_used=deny, expect_used=deny)
- Tauri **2.x** - 47 IPC commands (40 Alpha + 7 Beta 1)
- russh **0.56** - Pure Rust SSH with aws-lc-rs crypto
- bb8 **0.9** - Connection pooling (10x perf)
- sqlx **0.8** - SQLite with compile-time validation
- tokio **1.x** - Async multi-threaded runtime

**Development:**

- bun - Package manager
- ESLint **9.39.2** + Prettier **3.7.4**
- cargo clippy - Strict linting

**Critical Constraints:**

- ✅ Svelte 5.x ONLY (runes-based, NOT compatible with legacy stores)
- ✅ Rust 2021 (clippy lints require 2021 features)
- ✅ TypeScript strict mode (all strict flags enabled)
- ✅ Tauri 2.x (NOT compatible with Tauri 1.x IPC)

---

## Critical Implementation Rules

### Language-Specific Rules

**Rust Backend:**

**Error Handling (CRITICAL):**

- ❌ **NEVER use unwrap() or expect()** - Clippy denies with unwrap_used/expect_used=deny
- ✅ **ALWAYS use Result<T, String> with ? operator**
- ✅ **Use ok_or("message")? for Option types**
- ✅ **Provide context with map_err()**: `result.map_err(|e| format!("Context: {}", e))?`

**State Access:**

- ✅ **Lock → as_ref() → ok_or() → use**: `state.db.lock().await.as_ref().ok_or("Not initialized")?`
- ✅ **Clone for long-lived resources**: `.clone()` after getting reference
- ✅ **Always handle None explicitly** - No unwrapping

**Async Patterns:**

- ✅ **All Tauri commands must be async**: `#[tauri::command] async fn ...`
- ✅ **Always .await? for error propagation** - Don't use .await.unwrap()
- ✅ **Use tokio spawn for background tasks** - Not std::thread

**TypeScript/Svelte 5 Frontend:**

**Runes-Based State (CRITICAL):**

- ❌ **NEVER use legacy stores** (writable, readable, derived from 'svelte/store')
- ✅ **ALWAYS use Svelte 5 runes**:
  - `let x = $state(value)` for reactive state
  - `let y = $derived(expression)` for computed values
  - `$effect(() => { ... })` for side effects

**Component Props:**

- ✅ **Use $props() with TypeScript interface**:
  ```typescript
  interface Props {
    title: string;
    onClose?: () => void;
  }
  const { title, onClose }: Props = $props();
  ```

**Type Safety:**

- ❌ **NEVER use any type** - ESLint @typescript-eslint/no-explicit-any=error
- ✅ **All API calls must have explicit return types**
- ✅ **Import types from $lib/types.ts**

**API Calls:**

- ✅ **Always use centralized api.ts**: `import * as api from '$lib/api'`
- ✅ **All IPC calls wrapped in typed functions** - No raw invoke()

### Framework-Specific Rules

**Tauri 2 IPC:**

**Command Registration:**

- ✅ **Register all commands in lib.rs**: `tauri::generate_handler![...]`
- ✅ **Command naming**: snake_case verb_noun (get_queue_state, NOT getQueueState)
- ✅ **All commands async**: `#[tauri::command] async fn ...`
- ✅ **Command signature**: `async fn name(state: State<'_, AppState>) -> Result<T, String>`

**State Management:**

- ✅ **AppState pattern**: `Arc<Mutex<T>>` for shared state
- ✅ **Add to state.rs**: New managers (queue_manager, server_db)
- ✅ **Lock order**: Always lock in same order to prevent deadlocks

**Svelte 5 Component Patterns:**

**Component Organization:**

- ✅ **Feature-driven**: `lib/features/{feature}/` directory per feature
- ✅ **Component naming**: PascalCase (QueuePanel.svelte, QueueItem.svelte)
- ✅ **Colocation**: Keep related components together in feature folder

**Event Handling:**

- ✅ **Callbacks via props**: `onSubmit?: (data: T) => void`
- ✅ **Optional chaining**: `onSubmit?.(data)` to call nullable callbacks
- ✅ **No event bubbling tricks** - Explicit prop drilling

**Store Usage:**

- ✅ **Global stores in lib/stores/**: panels, shortcuts, toast, queue
- ✅ **Polling with $effect**: 2-second intervals for real-time updates
- ✅ **Derived state**: Use $derived for computed values

**Beta 1 Architecture (CRITICAL):**

**Module Isolation:**

- ❌ **NEVER modify existing Alpha modules** (db.rs, job.rs, ssh/, project.rs)
- ✅ **Create NEW isolated modules**: queue_service.rs, server_db.rs, reconciliation.rs, wrapper.rs
- ✅ **EXTEND existing modules**: Only add to lib.rs, commands.rs, state.rs

**Reconciliation Priority Chain:**

- ✅ **SQLite FIRST**: Always query server DB first
- ✅ **State file FALLBACK**: Check JSON state files if SQLite fails
- ✅ **tmux check INFERENCE**: Check session existence as last resort
- ✅ **ERROR on state loss**: Return clear error if all sources fail

**Wrapper Deployment:**

- ✅ **Embed with include_str!**: `include_str!("../scripts/job_wrapper.sh")`
- ✅ **Deploy to ~/.solverpilot/bin/**: SSH mkdir + write_file + chmod +x
- ✅ **Version tracking**: Include wrapper_version in DB and state files

**Data Separation:**

- ✅ **Local DB**: `~/.solverpilot/local.db` (client-only, projects/benchmarks)
- ✅ **Server DB**: `~/.solverpilot-server/server.db` (via SSH, job coordination)
- ❌ **NO shared files** - Always access server DB via SSH, never local file access

### Testing Rules

**Test Organization:**

- ✅ **Unit tests colocated**: `#[cfg(test)] mod tests` in each module
- ✅ **Integration tests in tests/**: queue_workflow, reconnect scenarios
- ✅ **Mock external dependencies**: SSH, tmux, SQLite for unit tests

**Priority Tests (Beta 1):**

**Reconciliation Logic (CRITICAL):**

- ✅ **Test priority chain**: SQLite > State File > tmux > Error
- ✅ **Test all scenarios**: completed, running, failed, crashed, orphaned
- ✅ **Mock SSH and tmux**: Use MockSshManager for predictable tests
- ✅ **Test error paths**: All fallback scenarios must be tested

**Queue State Machine:**

- ✅ **Test FIFO progression**: Jobs execute in order
- ✅ **Test sequential execution**: max_concurrent = 1 enforced
- ✅ **Test queue pausing**: Jobs stop after current completes
- ✅ **Test job cancellation**: Running job terminates cleanly

**Wrapper Script:**

- ✅ **Test deployment**: include_str! loads correct content
- ✅ **Test trap EXIT**: cleanup() called on all exit paths
- ✅ **Test SQLite updates**: Job status written correctly
- ✅ **Test state file writes**: JSON format matches schema

**Mock Patterns:**

```rust
// ✅ REQUIRED pattern for mocking
struct MockSshManager {
    responses: HashMap<String, String>,
}

impl MockSshManager {
    async fn exec(&self, cmd: &str) -> Result<String, String> {
        self.responses.get(cmd)
            .cloned()
            .ok_or_else(|| format!("No mock for: {}", cmd))
    }
}
```

**Test Requirements:**

- ✅ **All async tests use #[tokio::test]**
- ✅ **Use Result<(), Box<dyn std::error::Error>>** for test functions
- ✅ **No unwrap() in tests** - Use ? operator for clarity
- ✅ **Clear test names**: test_reconcile_completed_job_from_sqlite

**Coverage Goals (Beta 1.1):**

- Priority 1: Reconciliation logic (100% coverage)
- Priority 2: Queue state machine (90% coverage)
- Priority 3: Wrapper deployment (80% coverage)

### Code Quality & Style Rules

**Rust Backend:**

**Linting (Clippy Strict):**

- ✅ **Run before commit**: `cargo clippy` must pass with zero warnings
- ❌ **FORBIDDEN**: unwrap_used, expect_used (clippy denies)
- ⚠️ **Warn on**: dbg_macro, todo, unimplemented
- ✅ **Correctness/Suspicious**: Denied (compilation errors)

**Code Organization:**

- ✅ **One service per file**: queue_service.rs, server_db.rs, reconciliation.rs
- ✅ **Multi-file services use mod.rs**: ssh/mod.rs for ssh module
- ✅ **Commands in commands.rs**: All #[tauri::command] functions
- ✅ **State in state.rs**: AppState struct definition

**Naming Conventions:**

- Modules: `snake_case` (queue_service, server_db)
- Structs/Enums: `PascalCase` (QueueManager, JobStatus)
- Functions: `snake_case` (get_queue_state, reconcile_job)
- Constants: `SCREAMING_SNAKE_CASE` (MAX_RETRIES, WRAPPER_VERSION)

**Documentation:**

- ✅ **Public APIs need doc comments**: `/// Description`
- ✅ **Examples in doc comments**: For complex functions
- ❌ **No obvious comments**: Code should be self-explanatory

**TypeScript/Svelte Frontend:**

**Linting (ESLint + Prettier):**

- ✅ **Run before commit**: `bun run quality` must pass
- ❌ **FORBIDDEN**: any type (ESLint error)
- ✅ **REQUIRED**: await all promises (no floating promises)
- ✅ **Prettier format**: Always run `bun run format`

**Code Organization:**

- ✅ **Feature-driven**: `lib/features/{feature}/` per feature
- ✅ **Reusable in lib/ui/**: Button, Modal, Badge, Toast
- ✅ **Global stores in lib/stores/**: queue, panels, shortcuts
- ✅ **Types in types.ts**: All shared TypeScript interfaces

**Naming Conventions:**

- Components: `PascalCase.svelte` (QueuePanel, QueueItem)
- Stores: `camelCase.svelte.ts` (queue, panels)
- Types: `PascalCase` (QueueState, JobRecord)
- Functions: `camelCase` (getQueueState, reconcileJobs)
- Constants: `SCREAMING_SNAKE_CASE` (POLLING_INTERVAL_MS)

**Documentation:**

- ✅ **JSDoc for public functions**: Type annotations + description
- ✅ **Props interfaces**: Document each prop with JSDoc
- ❌ **No obvious comments**: Use descriptive names instead

**Quality Checks (Pre-Commit):**

**Backend:**

```bash
cargo clippy              # Zero warnings required
cargo fmt                 # Auto-format
cargo test                # All tests pass
```

**Frontend:**

```bash
bun run quality           # Lint + format + type-check
# Equivalent to:
# - bun run lint:check
# - bun run format:check
# - bun run check (svelte-check)
```

**Git Commit Requirements:**

- ✅ **Quality checks pass**: Both cargo clippy and bun run quality
- ✅ **No console.log in production**: Remove debug statements
- ✅ **No TODO comments**: Fix or create issue instead

### Development Workflow Rules

**Development Commands:**

**Daily Development:**

```bash
bun run tauri dev              # Hot-reload dev environment
```

**Quality Checks (Pre-Commit):**

```bash
bun run quality                # Frontend: lint + format + type-check
cargo clippy                   # Backend: zero warnings required
cargo fmt                      # Backend: auto-format
cargo test                     # Backend: all tests pass
```

**Git Workflow:**

**Commit Pattern:**

- ✅ **Run quality checks first**: `bun run quality && cargo clippy`
- ✅ **Descriptive commit messages**: Type + scope + description
- ✅ **Co-Authored-By footer**: `Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>`
- ❌ **DO NOT push** unless user explicitly requests it

**Commit Message Format:**

```
<type>(<scope>): <description>

- Bullet point change 1
- Bullet point change 2

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`  
Scopes: `queue`, `reconciliation`, `wrapper`, `frontend`, `backend`

**Branch Strategy:**

- `main` - Production code (Alpha + stable Beta)
- Feature branches - New development (feat/beta1-queue-management)

**Beta 1 Implementation Sequence:**

**Phase 1: Backend Foundation**

1. Create 4 NEW isolated modules (queue_service, server_db, reconciliation, wrapper)
2. Create wrapper script (job_wrapper.sh)
3. Write unit tests for each module

**Phase 2: Backend Integration**

1. Extend lib.rs (register 7 new commands)
2. Extend commands.rs (implement 7 new commands)
3. Extend state.rs (add queue_manager, server_db)
4. Extend ssh/executor.rs (add wrapper invocation)

**Phase 3: Frontend Feature**

1. Create queue feature module (lib/features/queue/)
2. Create 5 new Svelte components
3. Create queue store (lib/stores/queue.svelte.ts)
4. Add types to types.ts, API wrappers to api.ts

**Phase 4: Integration & Testing**

1. Extend MainLayout.svelte (add QueuePanel slot)
2. Test end-to-end (queue → execute → disconnect → reconnect → reconcile)
3. Run all quality checks

**Hot-Reload Behavior:**

- **Frontend-only changes**: Instant HMR (Vite)
- **Backend changes**: 2-5s Cargo recompile + Tauri reload
- **Both changed**: Backend compiles first, then frontend HMR

**Build & Deployment:**

**Development Build:**

```bash
bun run tauri dev              # Debug mode (fast compile, debug=1)
```

**Production Build:**

```bash
bun run tauri build            # Release mode (LTO, strip, opt-level=z)
# Outputs: .deb, .AppImage (Linux), .dmg (macOS), .msi (Windows)
```

**Pre-Release Checklist:**

- ✅ All quality checks pass (frontend + backend)
- ✅ All tests pass (cargo test)
- ✅ Architecture document reviewed
- ✅ No unwrap/expect violations (cargo clippy)
- ✅ No console.log in production code
- ✅ Wrapper script version updated if modified

### Critical Don't-Miss Rules

**❌ FORBIDDEN Anti-Patterns:**

**Rust Backend:**

- ❌ **NEVER use unwrap() or expect()** - Clippy denies, use ok_or()? instead
- ❌ **NEVER modify Alpha modules** - db.rs, job.rs, ssh/, project.rs are READ-ONLY
- ❌ **NEVER access server DB via local file** - Always via SSH (security + network transparency)
- ❌ **NEVER use std::thread** - Use tokio::spawn for async tasks
- ❌ **NEVER skip reconciliation priority** - Always: SQLite → State File → tmux → Error

**TypeScript/Svelte Frontend:**

- ❌ **NEVER use legacy stores** - Use Svelte 5 runes ($state, $derived, $effect)
- ❌ **NEVER use any type** - ESLint error, always explicit types
- ❌ **NEVER use raw invoke()** - Use typed wrappers from api.ts
- ❌ **NEVER skip await** - Floating promises are ESLint errors

**Edge Cases (MUST Handle):**

**Reconciliation Scenarios:**

1. **Wrapper crashed before status write** → State file missing + tmux exists → Running
2. **SQLite unavailable (disk full)** → Fallback to state file, log error
3. **Orphaned tmux session** → DB=completed + tmux exists → Trust DB (completed)
4. **Network partition mid-job** → Reconnect reconciliation → Resume from server state
5. **SIGKILL on wrapper (rare)** → trap EXIT doesn't fire → Detect missing state → Mark crashed

**Queue Edge Cases:**

1. **Queue operation during reconciliation** → Block with "Syncing..." toast (5-10s)
2. **Cancel running job** → tmux kill-session → Update DB "killed" → Continue queue
3. **Pause with running job** → Wait for current completion → Don't start next
4. **Connection lost mid-queue** → Startup reconciliation → Continue from stopped point

**SSH Connection Edge Cases:**

1. **ControlMaster stale** → bb8 auto-recreates → Retry command
2. **Network timeout** → Exponential backoff (1s, 2s, 4s, 8s) → Max 5 retries
3. **Server reboot** → All tmux lost → Reconciliation marks crashed

**Security (CRITICAL):**

**Credential Handling:**

- ✅ **SSH keys ONLY** - Never store passwords in any form
- ✅ **Redact logs** - No credentials in tracing output
- ✅ **File permissions** - 0600 for keys, 0644 for state files
- ❌ **Never commit secrets** - .gitignore for config with credentials

**SQL Injection:**

- ✅ **Escape single quotes** - `job_id.replace("'", "''")`
- ✅ **Validate UUIDs** - Check format before SQL injection
- ❌ **No string interpolation** - Especially for user-provided input

**Performance Gotchas:**

1. **Polling interval** - NEVER faster than 2 seconds (SSH exhaustion)
2. **Query optimization** - NEVER query all jobs (use WHERE status IN ('queued', 'running'))
3. **Lock duration** - NEVER hold lock during .await (deadlock risk)
4. **Connection pooling** - ALWAYS use bb8 pool (10x faster than new connections)
5. **State file size** - Keep JSON minimal (<1KB per file)

**Critical Success Paths:**

**Happy Path (Queue → Execute → Complete):**

1. Frontend queues jobs → Backend inserts to server DB (status=queued)
2. Queue auto-starts next job → Deploy wrapper if needed
3. Wrapper updates DB (status=running) → Python executes
4. Wrapper trap EXIT fires → Updates DB (status=completed, exit_code=0)
5. Frontend polling detects → Updates UI → Auto-starts next job

**Disconnect/Reconnect Path:**

1. Connection lost mid-job → Job continues on server
2. User reconnects → reconcile_all_jobs() called
3. Reconciliation finds completed job → Shows resume modal
4. User clicks "Resume Queue" → Queue continues from next job

**Failure Recovery Path:**

1. Job fails (exit_code ≠ 0) → Wrapper writes DB (status=failed)
2. Queue continues to next job → Failed job preserved in history
3. User clicks "Retry" on failed job → Re-queues with same ID
4. Job re-executes → New attempt tracked in logs

---

## Quick Reference for AI Agents

**Before implementing ANY code, review:**

1. ✅ Technology stack versions (Svelte 5 runes, Rust 2021, Tauri 2)
2. ✅ Error handling patterns (Result<T, String>, no unwrap/expect)
3. ✅ Module isolation (NEW Beta 1 modules, preserve Alpha)
4. ✅ Reconciliation priority chain (SQLite → File → tmux → Error)
5. ✅ Quality checks (cargo clippy, bun run quality)

**Common mistakes to avoid:**

- ❌ Using legacy Svelte stores instead of runes
- ❌ Using unwrap() or expect() (clippy denies)
- ❌ Modifying existing Alpha modules
- ❌ Accessing server DB via local file
- ❌ Skipping reconciliation priority chain

**For questions, refer to:**

- Full architecture: `_bmad-output/planning-artifacts/architecture.md` (6000+ lines)
- Existing code patterns: `CLAUDE.md`
- Project documentation: `docs/index.md`

---

## Quick Reference for AI Agents

**Before implementing ANY code, review:**

1. ✅ **Technology Stack** - Svelte 5 runes ($state/$derived/$effect), Rust 2021, Tauri 2.x, russh 0.56, bb8 0.9
2. ✅ **Error Handling** - Result<T, String> always, NEVER unwrap()/expect() (clippy denies)
3. ✅ **Module Isolation** - NEW modules (queue_service, server_db, reconciliation, wrapper), EXTEND (lib.rs, commands.rs, state.rs), PRESERVE Alpha (db.rs, job.rs, ssh/)
4. ✅ **Reconciliation Priority** - SQLite FIRST → State File FALLBACK → tmux INFERENCE → ERROR
5. ✅ **Quality Checks** - `cargo clippy` (zero warnings) + `bun run quality` (lint, format, type-check)

**For questions, refer to:**

- **Architecture Details**: `_bmad-output/planning-artifacts/architecture.md` (6000+ lines, 10 decisions, 216 FRs)
- **General Patterns**: `CLAUDE.md` (development commands, architecture overview)
- **Project Structure**: `docs/index.md` (28+ components, 40+ commands, complete inventory)

**Critical Reminders:**

- ❌ **NEVER** modify Alpha modules (db.rs, job.rs, ssh/, project.rs, python_deps.rs)
- ❌ **NEVER** use legacy Svelte stores (writable, readable, derived)
- ❌ **NEVER** access server DB via local filesystem (always SSH)
- ✅ **ALWAYS** test reconciliation scenarios (disconnect, reconnect, sync)
- ✅ **ALWAYS** run quality checks before committing

**Edge Case Checklist:**

- [ ] Wrapper crash + state file missing + tmux exists → Status: Running
- [ ] SQLite unavailable → Fallback to state file parsing
- [ ] Network partition during job → Reconciliation resumes from server state
- [ ] Queue operation during reconciliation → Block with "Syncing..." toast
- [ ] ControlMaster stale → bb8 pool auto-recreates connection
