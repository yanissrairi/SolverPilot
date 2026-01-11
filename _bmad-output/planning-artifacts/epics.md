---
stepsCompleted: [1, 2, 3, 4]
status: 'complete'
completedAt: '2026-01-11'
inputDocuments:
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
  - '_bmad-output/planning-artifacts/ux-design-specification.md'
  - '_bmad-output/planning-artifacts/research/technical-remote-job-state-capture-ssh-tmux-research-2026-01-08.md'
totalEpics: 6
totalStories: 28
totalFRsCovered: 54
validationStatus: 'all-checks-passed'
---

# SolverPilot - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for SolverPilot, decomposing the requirements from the PRD, UX Design, Architecture, and Research documents into implementable stories.

## Requirements Inventory

### Functional Requirements

**Total: 216 Requirements across 8 Capability Areas**

#### Configuration & Setup (38 FRs: FR1-FR38)

**Core Configuration:**

- FR1: User can configure remote server connection settings (hostname, username, SSH key path, remote base directory)
- FR2: User can specify Python version for remote environment
- FR3: User can test SSH connection before saving configuration
- FR4: User can save validated connection configuration
- FR5: User can edit existing server configuration
- FR6: User can view current configuration settings

**SSH Key Management:**

- FR7: User can select SSH private key file via file picker
- FR8: System can load SSH key from standard locations (~/.ssh/)
- FR9: User can provide passphrase for encrypted SSH keys
- FR10: System can validate SSH key format before saving
- FR11: User can switch between multiple SSH keys
- FR12: System can handle SSH agent integration for key authentication

**Connection Testing & Validation:**

- FR13: User can test connection with real-time feedback
- FR14: System can validate hostname reachability before saving
- FR15: System can verify remote base directory exists and is writable
- FR16: System can detect SSH connection failures with actionable error messages
- FR17: User can retry failed connection tests without re-entering configuration

**Configuration Persistence:**

- FR18: System can save configuration to platform-specific config directory
- FR19: System can load configuration on application startup
- FR20: System can detect missing or corrupted configuration
- FR21: User can reset configuration to defaults
- FR22: System can backup configuration before changes

**Multi-Server Support:**

- FR23: User can configure multiple remote servers
- FR24: User can switch active server from UI
- FR25: System can warn when switching servers with pending jobs
- FR26: User can name server configurations for easy identification
- FR27: System can prevent duplicate server configurations

**Advanced Connection Settings:**

- FR28: User can configure custom SSH port (non-standard)
- FR29: User can enable SSH connection through proxy/jump host
- FR30: User can configure connection timeout values
- FR31: User can enable verbose SSH logging for debugging
- FR32: System can support SSH config file integration (~/.ssh/config)

**Security & Credentials:**

- FR33: System can protect SSH credentials in memory (never exposed to frontend)
- FR34: System can zeroize sensitive data after use
- FR35: User can configure two-factor authentication handling
- FR36: System can handle SSH certificate-based authentication
- FR37: User can disable strict host key checking (development mode)
- FR38: System can remember known hosts across sessions

#### Project Management (13 FRs: FR39-FR54)

**Core Project Operations:**

- FR39: User can create new Python project with name and Python version
- FR40: User can select local source directory for project
- FR41: User can list all projects
- FR42: User can delete project and associated data
- FR43: User can rename existing project
- FR44: User can view project metadata (created date, Python version, benchmark count)

**Project State & Tracking:**

- FR45: System can mark active/current project in UI
- FR46: User can switch between multiple projects
- FR47: System can track which project was last used
- FR48: User can view project history (jobs run, success rate)

**Git Integration:**

- FR49: System can detect if project directory is Git repository
- FR50: User can view current Git branch for project
- FR51: System can warn about uncommitted changes before job execution

**Project Portability:**

- FR52: User can export project configuration
- FR53: User can import project from configuration file
- FR54: System can validate project directory structure on import

#### Benchmark Management (16 FRs: FR55-FR72)

**Benchmark File Management:**

- FR55: User can add Python file as benchmark via file picker
- FR56: User can remove benchmark from project
- FR57: User can view list of all benchmarks in project
- FR58: User can see benchmark status (ready, needs dependencies, error)

**Benchmark Organization:**

- FR59: User can organize benchmarks into folders/categories
- FR60: User can search/filter benchmarks by name
- FR61: User can tag benchmarks with custom labels
- FR62: User can sort benchmarks (alphabetically, by last run, by status)

**Benchmark Metadata:**

- FR63: System can display file size and last modified date for benchmarks
- FR64: User can view benchmark dependency list without running
- FR65: System can detect changes to benchmark files (modification tracking)

**Benchmark Templates & Duplication:**

- FR66: User can duplicate benchmark with new name
- FR67: User can create benchmark from template
- FR68: User can mark benchmark as template for future use

**Hierarchical Organization:**

- FR69: User can create folder hierarchies for benchmarks
- FR70: User can move benchmarks between folders
- FR71: System can display benchmarks in tree view
- FR72: User can collapse/expand folders in benchmark list

#### Dependency Management (18 FRs: FR73-FR94)

**Core Dependency Detection:**

- FR73: System can analyze Python file imports using tree-sitter AST parsing
- FR74: System can distinguish between external packages and local file dependencies
- FR75: System can detect import statements (import, from...import, multiline imports)
- FR76: System can identify standard library vs third-party packages
- FR77: System can build dependency tree for benchmark (recursive local imports)

**Dependency Installation:**

- FR78: System can install external packages via uv automatically
- FR79: System can check existing environment before installing packages
- FR80: System can skip already-installed packages
- FR81: System can detect package installation failures with error messages
- FR82: User can manually add packages to project dependencies

**Validation & Overrides:**

- FR83: User can override automatic dependency detection
- FR84: User can mark benchmark as "dependencies verified" to skip re-checking
- FR85: System can re-validate dependencies when Python file changes

**Version Conflict Handling:**

- FR86: System can detect dependency version conflicts
- FR87: User can resolve conflicts by specifying version constraints
- FR88: System can warn about incompatible package versions before job execution

**Advanced Dependency Scenarios:**

- FR89: System can handle private package repositories (credentials required)
- FR90: User can configure custom package index URLs
- FR91: System can detect system-level dependencies (non-Python, e.g., CUDA, MPI)
- FR92: User can specify additional pip install flags for complex packages
- FR93: System can handle circular dependency detection and warnings
- FR94: System can support editable/development package installations

#### Job Execution & Experiment Management (24 FRs: FR95-FR121)

**Core Job Execution:**

- FR95: User can run single benchmark with one-click
- FR96: System can rsync project files to remote server automatically
- FR97: System can use SSH connection pool for efficient connections
- FR98: System can create unique tmux session for each job
- FR99: System can start Python script in tmux session
- FR100: System can return immediately after starting job (non-blocking)

**Command-Line Arguments:**

- FR101: User can specify command-line arguments for Python script
- FR102: User can save argument presets for frequently-used configurations
- FR103: User can select from saved presets when launching job
- FR104: System can validate argument syntax before execution

**Environment Variables:**

- FR105: User can set environment variables for job execution
- FR106: User can save environment variable presets
- FR107: System can pass environment variables to remote Python process

**Dry Run & Testing:**

- FR108: User can perform dry-run (show what will execute without running)
- FR109: System can preview rsync operations before execution
- FR110: User can test Python syntax remotely without full execution

**Argument Safety & Validation:**

- FR111: System can detect potentially dangerous shell characters in arguments
- FR112: User can escape special characters in command-line arguments
- FR113: System can preview full command before execution

**Pre-Flight Checks:**

- FR114: System can check remote disk space before rsync
- FR115: System can detect tmux session name conflicts
- FR116: System can verify remote Python version matches project requirements
- FR117: System can check remote file permissions before job execution
- FR118: User can override pre-flight check failures (acknowledge risks)

**Job Cancellation:**

- FR119: User can cancel running job
- FR120: System can kill tmux session cleanly
- FR121: System can update job status to "cancelled" in database

#### Job Monitoring & Status (20 FRs: FR122-FR147)

**Real-Time Monitoring:**

- FR122: User can view live log output from running job
- FR123: System can stream logs from remote tmux session
- FR124: System can parse progress indicators (e.g., [x/y] patterns)
- FR125: System can display elapsed time with live counter
- FR126: System can detect job completion (exit patterns)
- FR127: System can show final job status (success, failure, cancelled)

**Offline & Resilience:**

- FR128: User can view job status when network is offline (from local database)
- FR129: System can reconnect to running job after network interruption
- FR130: User can check status of job started in previous session

**Complete Logs & Export:**

- FR131: User can view complete log history for finished jobs
- FR132: User can export logs to text file
- FR133: System can preserve logs in database for historical analysis

**Enhanced Monitoring:**

- FR134: User can see which job is currently running across all projects
- FR135: System can detect job timeout (configurable max execution time)
- FR136: System can warn about abnormally long-running jobs
- FR137: User can receive notifications for job state changes

**Server Resource Monitoring:**

- FR138: System can monitor remote server CPU usage during job execution
- FR139: System can monitor remote server memory usage
- FR140: System can warn when server resources are constrained

**Custom Progress Parsing:**

- FR141: User can define custom progress patterns for specific benchmarks
- FR142: System can extract iteration count, loss values, or custom metrics from logs
- FR143: User can visualize custom metrics in real-time

**Progress Validation & Alerts:**

- FR144: System can detect stalled jobs (no progress updates for threshold time)
- FR145: User can configure alerts for specific log patterns (errors, warnings)
- FR146: System can detect runaway jobs (excessive output, memory leaks)
- FR147: User can set job execution time limits with automatic cancellation

#### Queue Management (Beta 1) (36 FRs: FR148-FR191)

**Core Queue Operations:**

- FR148: User can select multiple benchmarks for queueing (shift-click, ctrl-click)
- FR149: User can add selected benchmarks to queue with one action
- FR150: System can store queue in SQLite database
- FR151: User can view all jobs in queue (pending, running, completed, failed)
- FR152: User can start queue processing
- FR153: User can pause queue (finish current job, stop starting new ones)
- FR154: User can resume paused queue
- FR155: System can process queue sequentially (one job at a time)
- FR156: System can automatically start next pending job when current completes
- FR157: User can cancel all pending jobs in queue
- FR158: User can remove specific job from queue (before execution)

**Queue Persistence & Recovery:**

- FR159: System can persist queue across application restarts
- FR160: User can resume queue processing after reopening application
- FR161: System can recover from SolverPilot crashes without losing queue state
- FR162: System can detect partially-completed queues on startup
- FR163: User can view queue recovery status with clear indicators

**Enhanced Queue Control:**

- FR164: User can reorder jobs in queue (drag-and-drop or priority numbers)
- FR165: User can move job to front of queue
- FR166: User can move job to end of queue
- FR167: System can show queue position for each pending job
- FR168: System can estimate time remaining for queue (based on average job duration)
- FR169: User can filter queue view (show only pending, only failed, etc.)

**Job Scheduling:**

- FR170: User can schedule job to start at specific time
- FR171: System can delay job execution until scheduled time
- FR172: User can cancel scheduled jobs before they start

**Duplicate Detection & Management:**

- FR173: System can detect duplicate jobs in queue (same benchmark, same arguments)
- FR174: User can configure duplicate handling (allow, warn, prevent)
- FR175: System can warn when adding job that's already in queue
- FR176: User can replace existing queued job with new configuration

**Schedule Integration:**

- FR177: User can schedule entire queue to start at specific time
- FR178: User can create recurring queue schedules (daily, weekly)
- FR179: System can execute queues on schedule without manual start

**Audit Log & History:**

- FR180: System can log all queue operations (add, remove, reorder, start, pause)
- FR181: User can view queue operation history
- FR182: System can timestamp all queue state changes
- FR183: User can filter audit log by operation type or date range

**Failure Handling & Robustness:**

- FR184: System can prevent cascade failures (failed job doesn't stop queue)
- FR185: User can configure retry behavior for failed jobs
- FR186: System can automatically retry failed jobs with exponential backoff
- FR187: System can auto-reconnect SSH after transient connection failures
- FR188: System can clean up orphaned tmux sessions from previous crashes
- FR189: System can detect server reboots and re-establish connections
- FR190: User can view failure reasons for all failed jobs in queue
- FR191: System can quarantine repeatedly-failing jobs (prevent infinite retries)

#### Result Management (Beta 2) (51 FRs: FR192-FR252)

**Core Result Download:**

- FR192: User can configure output file patterns per project (e.g., results/\*.csv)
- FR193: System can detect files matching patterns after job completion
- FR194: System can download result files from remote server to local machine
- FR195: System can organize downloads by project/job/benchmark structure
- FR196: System can handle various file sizes (1KB to 1GB+)
- FR197: System can show download progress for large files

**Desktop Notifications:**

- FR198: System can send desktop notification when job completes
- FR199: System can send notification when result download completes
- FR200: System can send notification when queue finishes processing
- FR201: System can send notification for critical errors (SSH connection lost, job failure)
- FR202: User can enable/disable notifications in settings
- FR203: User can configure notification sounds
- FR204: User can click notification to open SolverPilot and view job

**Pattern Management:**

- FR205: User can specify multiple output patterns per project
- FR206: User can preview files that match pattern before downloading
- FR207: User can test pattern against remote directory structure
- FR208: System can support glob patterns (_.csv, output/\*\*/_.json)
- FR209: User can exclude specific files from download (e.g., _.tmp, _.log)

**Local File Operations:**

- FR210: User can configure local download location (project-organized or custom)
- FR211: User can open download location in file explorer
- FR212: System can preserve remote directory structure in local downloads
- FR213: User can flatten directory structure (all files in one folder)

**Download Validation & Integrity:**

- FR214: System can verify downloaded file sizes match remote files
- FR215: System can detect corrupted downloads (checksum validation)
- FR216: User can retry failed downloads
- FR217: System can resume partial downloads after network interruption
- FR218: System can warn about file size mismatches

**Notification & Storage Management:**

- FR219: User can set download size limits (warn on >100MB files)
- FR220: System can estimate local disk space required before downloading
- FR221: User can approve/reject large downloads before transfer
- FR222: System can clean up old downloads automatically (configurable retention)
- FR223: User can archive or delete downloads from UI

**Comparative Analysis & Annotations:**

- FR224: User can compare results across multiple job runs
- FR225: System can display result file previews (CSV tables, JSON structure)
- FR226: User can annotate job results with notes
- FR227: User can tag results for organization
- FR228: User can export result metadata to spreadsheet

**Download Metadata Management:**

- FR229: System can track download history (filename, size, timestamp, source job)
- FR230: User can search download history
- FR231: System can link downloaded files to originating job/benchmark

**Download Reliability:**

- FR232: System can handle simultaneous downloads from queue completion
- FR233: System can throttle download bandwidth (configurable)
- FR234: System can pause/resume downloads
- FR235: System can retry downloads with exponential backoff
- FR236: User can manually trigger re-download of job results

**Multi-Server Result Notifications:**

- FR237: System can group notifications by server when using multi-server setup
- FR238: User can configure per-server notification preferences
- FR239: System can distinguish notification sources (Server A vs Server B)

**Result File Metadata & Formats:**

- FR240: User can specify expected result file format (CSV, JSON, HDF5, images)
- FR241: System can validate file format after download
- FR242: User can configure post-download file transformations (compression, renaming)

**Download Safety & Edge Cases:**

- FR243: System can prevent downloading excessive number of files (configurable limit)
- FR244: System can detect and handle symbolic links on remote server
- FR245: System can check local disk space before initiating download
- FR246: System can warn when downloading from actively-running job
- FR247: User can configure download behavior for in-progress jobs
- FR248: System can handle remote file deletions gracefully (warn user)
- FR249: System can detect network proxy interference with downloads
- FR250: User can configure download timeout values
- FR251: System can handle file permission issues on remote server
- FR252: System can support downloading from different remote paths than execution path

### NonFunctional Requirements

**Total: 62 NFRs**

#### Performance (9 NFRs)

- NFR-P1: SSH connection establishment must complete within 5 seconds under normal network conditions (<50ms latency)
- NFR-P2: Connection pool reuse must reduce subsequent SSH operations to <500ms (eliminating handshake overhead)
- NFR-P3: File sync operations (rsync) must provide real-time progress feedback for transfers >10MB
- NFR-P4: All UI interactions must respond within 100ms (button clicks, navigation, input)
- NFR-P5: Background SSH operations must not block UI thread (async execution mandatory via Tokio runtime)
- NFR-P6: Job status polling must update UI within 2 seconds of remote state change
- NFR-P7: tree-sitter AST parsing must complete within 2 seconds for Python files <5000 lines
- NFR-P8: Dependency analysis must handle recursive imports up to 10 levels deep
- NFR-P9: CI/CD pipeline must include performance benchmarks for SSH connection pooling and dependency analysis with <10% regression tolerance between releases

#### Reliability (12 NFRs)

- NFR-R1: SolverPilot must have zero known bugs in core workflows (SSH connection, file sync, job submission, queue management, monitoring) at time of release
- NFR-R2: Job submission must succeed 100% when server is reachable and credentials are valid
- NFR-R3: Connection pool must gracefully handle transient network issues with retry logic (3 retries with exponential backoff) before surfacing error to user
- NFR-R4: Queue persistence must survive application crashes without data loss (SQLite ACID properties)
- NFR-R5: SolverPilot failures must be clearly distinguished from Python script failures, server issues, or network problems
- NFR-R6: All error messages must be actionable (tell user what went wrong and how to fix it)
- NFR-R7: SQLite database operations must complete atomically (no partial writes on failure)
- NFR-R8: Job status tracking must accurately reflect remote job state (no phantom "running" jobs)
- NFR-R9: File sync must detect obvious transfer failures (size mismatches, connection drops, non-zero rsync exit codes)
- NFR-R10: tmux sessions must survive SSH disconnections without job termination
- NFR-R11: Application must reconnect to existing tmux sessions after network interruption
- NFR-R12: Queue processing must resume correctly after application restart

#### Security (12 NFRs)

- NFR-S1: SSH private keys must never be exposed to frontend (Rust backend only)
- NFR-S2: SSH key passphrases must be zeroized in memory immediately after use
- NFR-S3: Configuration file must use restrictive file permissions (0600 on Unix, ACL-protected on Windows)
- NFR-S4: No credentials may be logged to console or debug output
- NFR-S5: SQLite database must be stored in platform-specific user config directory (not world-readable)
- NFR-S6: Connection configuration must not include passwords (SSH key authentication only)
- NFR-S7: Sensitive data must not be transmitted to any external service (100% local-first architecture)
- NFR-S8: All user-provided paths must be sanitized to prevent directory traversal attacks
- NFR-S9: SSH command arguments must be escaped to prevent command injection
- NFR-S10: SQL queries must use parameterization to prevent SQL injection
- NFR-S11: SSH connections must verify host keys (prevent man-in-the-middle attacks)
- NFR-S12: SSH protocol must use secure algorithms (Ed25519/RSA keys, no legacy DSA)

#### Usability (17 NFRs)

- NFR-U1: Setup wizard must complete in <5 minutes for users with pre-existing SSH key pairs and basic SSH knowledge
- NFR-U2: Connection test must provide clear success/failure feedback with specific error messages
- NFR-U3: First-run experience must guide user through server configuration with validation at each step
- NFR-U4: All error messages must be user-friendly (no raw technical stack traces shown)
- NFR-U5: Error messages must suggest corrective actions
- NFR-U6: Toast notifications must auto-dismiss after 5-8 seconds for non-critical messages
- NFR-U7: Critical errors must persist until user acknowledgment
- NFR-U8: Long-running operations (>3 seconds) must show progress indicators
- NFR-U9: Job monitoring must update at minimum every 2 seconds during execution
- NFR-U10: Queue must show estimated time remaining based on job history
- NFR-U11: UI must follow platform conventions (Windows/macOS/Linux native patterns)
- NFR-U12: Primary workflows must be discoverable without documentation (intuitive controls)
- NFR-U13: Tooltips or help text must explain non-obvious features on hover
- NFR-U14: Application must behave identically across Windows, macOS, and Linux
- NFR-U15: File path handling must correctly process spaces and unicode characters
- NFR-U16: Application must clearly indicate network connectivity status and disable/grey-out network-dependent actions when offline
- NFR-U17: Application must handle platform-specific filesystem differences with clear error messages when unsupported scenarios detected

#### Maintainability (12 NFRs)

- NFR-M1: Rust code must pass clippy pedantic linting with zero warnings
- NFR-M2: No `unwrap()` or `expect()` in production code (explicit error handling required)
- NFR-M3: TypeScript strict mode must be enabled with zero `any` types
- NFR-M4: All public APIs must have inline documentation
- NFR-M5: CI/CD pipeline must build successfully for all supported platforms
- NFR-M6: Production builds must complete within 20 minutes in CI/CD
- NFR-M7: Installer artifacts must be <25MB per platform
- NFR-M8: Security audits (cargo-deny) must pass with zero known vulnerabilities
- NFR-M9: Dependencies must be locked (Cargo.lock, bun.lockb) for reproducible builds
- NFR-M10: Core workflows must have automated integration tests that run in CI/CD
- NFR-M11: Application must log detailed error context to platform-specific log directory for debugging
- NFR-M12: Application must detect and migrate data from previous versions without data loss during upgrades

### Additional Requirements

#### From Architecture Document

**Starter Template:** NO - SolverPilot Beta 1 is a **brownfield enhancement** building on existing Alpha foundation (40+ IPC commands, 28+ components, production-ready stack). NOT starting from greenfield template.

**Infrastructure & Deployment:**

- russh 0.56 + russh-keys 0.49 (pure Rust SSH with aws-lc-rs crypto)
- bb8 0.9 (async connection pooling - 10x performance improvement)
- SQLx 0.8 with SQLite (compile-time checked queries)
- Tauri 2.x desktop framework
- Svelte 5.0.0 with runes (NOT legacy stores)
- TypeScript 5.6.0 strict mode
- TailwindCSS 4.1.18 with @theme directive

**Integration Requirements:**

- Preserve all 40+ existing Alpha Tauri IPC commands
- SSH ControlMaster multiplexing for ~10x connection speed improvement
- Connection pool (bb8) integration with existing SSH manager
- 2-second backend polling cycle for queue state updates
- Connection health checks every 10 seconds
- 3-panel layout preservation (Left: Benchmarks, Center: Queue, Right: Logs)

**Data Migration & Setup:**

- ADD 3 new columns to existing `jobs` table: `tmux_session_name TEXT UNIQUE`, `started_at TEXT`, `completed_at TEXT`
- NO schema rewrite - ALTER TABLE only
- Foreign key constraints preserved
- Cascade deletes maintained

**Monitoring & Logging:**

- 2-second polling for queue state (status changes only, not full logs)
- 10-second lightweight health checks (echo command via SSH)
- Client-side elapsed time counters (reduce backend load)
- Progress parsing only when `[x/y]` regex matches new log lines
- tracing + tracing-subscriber structured logging

**API Versioning & Compatibility:**

- All new commands follow existing IPC pattern: `#[tauri::command] async fn name(state: State<'_, AppState>) -> Result<T, String>`
- JSON serialization via serde (all types derive Serialize/Deserialize)
- 8-12 NEW queue commands to add: `queue_benchmarks`, `start_queue`, `pause_queue`, `retry_job`, `reconcile_queue_state`, `get_queue_summary`, `get_connection_health`, `kill_job`, `clear_completed_jobs`
- Backward compatible with Alpha - no breaking changes to existing commands

**Security Implementation:**

- Rust clippy **denies** `unwrap_used` and `expect_used` (enforced at compile time)
- All error handling uses `Result<T, String>` with `?` operator or `.ok_or()`
- zeroize for secure memory wiping of SSH keys and credentials
- No credentials in SQLite database
- No telemetry or external data transmission (100% local-first)

#### From UX Design Document

**Responsive Design:**

- Desktop-only: Minimum window width 1024px (no mobile responsive design)
- Panel minimum widths: Left 200px, Middle 400px, Right 200px
- No touch gestures, no mobile breakpoints

**Accessibility Requirements:**

- WCAG AAA compliance: 12.6:1 contrast ratio minimum
- Minimum font size: 14px (text-sm) for all user-facing content
- Triple encoding: Status communicated via color + icon + text (never color alone)
- Focus visible states: `ring-2 ring-blue-500 ring-offset-2` on keyboard focus
- ARIA labels for status badges and screen reader announcements

**Browser/Device Compatibility:**

- Linux, macOS, Windows desktop environments
- Tauri 2 native desktop app (NOT browser-based)
- No browser compatibility concerns (uses system WebView)

**User Interaction Patterns:**

- **Keyboard shortcuts**: Q (queue selected), R (retry failed), Space (toggle selection), Shift-click (range select), Ctrl/Cmd-click (individual select)
- **Multi-select patterns**: Familiar file manager interactions
- **Drag-and-drop**: Job reordering in queue
- **Glassmorphism**: Differentiated blur radius (2px panels, 12px header) for 60fps resize performance

**Animation & Transition Requirements:**

- Smooth benchmark transition from left panel to center queue when queued
- Status badge color transitions
- Progress bar animations with `[x/y]` text
- Toast notification slide-in/slide-out
- Panel opacity hierarchy for visual depth (85%/75%/80%)

**Error Handling UX:**

- Failed jobs persist with red status badges (don't disappear)
- Raw solver output in logs (no prettified summaries)
- Error messages extracted from last 20 lines of log
- Queue continues executing other jobs (proves resilience)
- One-click retry with `R` keyboard shortcut

#### From Research Document (Technical Decisions)

**Remote Job State Capture Solution: Hybrid Approach**

- **Bash wrapper script** with `trap EXIT` for guaranteed cleanup (except SIGKILL)
- **SQLite server database** at `~/.solverpilot-server/server.db` (primary source of truth)
- **JSON state files** at `~/.solverpilot-server/jobs/<job_id>.status` (fallback mechanism)
- **flock** for atomic write guarantees

**Server-Side Database Schema:**

```sql
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    user TEXT NOT NULL DEFAULT 'default',
    benchmark_path TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('queued', 'running', 'completed', 'failed', 'killed')),
    tmux_session_name TEXT UNIQUE,
    queued_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    exit_code INTEGER,
    error_message TEXT,
    log_file TEXT,
    progress_current INTEGER,
    progress_total INTEGER
);
```

**Wrapper Script Deployment:**

- Deploy wrapper to `~/.solverpilot/bin/job_wrapper.sh` on remote server
- Wrapper embedded in Rust binary via `include_str!("../scripts/job_wrapper.sh")`
- Version tracking in database for debugging

**Reconciliation Priority Chain:**

1. **SQLite Database** (primary) - Query server.db for job status
2. **State Files** (fallback) - Parse JSON state file if SQLite unavailable
3. **Tmux Session Check** (inference) - Check if tmux session exists → status="running"
4. **Error State** (last resort) - Mark job as "state lost" if all sources fail

**Failure Modes & Edge Cases:**

- **SIGKILL**: trap EXIT won't run → Reconciliation detects missing state
- **SQLite corruption**: Automatic fallback to state files
- **Disk full**: trap EXIT logs error to stderr, limited recovery options
- **Network disconnect**: Wrapper continues on server, state written locally
- **Server reboot**: tmux session lost, reconciliation marks "failed"
- **Race conditions**: flock guarantees atomic writes, SQLite handles concurrency with WAL mode

### FR Coverage Map

_This section will be populated in Step 2 after epic design, mapping each FR to specific epics and stories._

## Epic List

_This section will be populated in Step 2 after epic design is complete._

## Epic List

### Epic 1: Queue Foundation & Multi-Job Submission

**User Outcome:** Users can select multiple benchmarks, add them to a queue, organize the queue, and view all queued jobs with their statuses.

**Value Delivered:** Users can organize their benchmark work into a unified queue instead of running jobs one-at-a-time. They can batch 10, 20, or 50 benchmarks before lunch and prepare for execution.

**FRs Covered:** FR148-FR158, FR164-FR169, FR173-FR176 (Total: 20 FRs)

**Key Capabilities:**

- Multi-select benchmarks (shift-click, ctrl-click, keyboard shortcuts)
- Add selected benchmarks to queue with one action (Q key or "Queue Selected" button)
- View all jobs in queue (pending, running, completed, failed) in center panel
- Remove specific jobs from queue before execution
- Reorder jobs (drag-and-drop, move to front/end, priority numbers)
- Queue position indicators (show #1, #2, #3 for pending jobs)
- Duplicate detection and management (warn/prevent/allow duplicates)
- Queue filtering (show only pending, only failed, etc.)

**Architecture Notes:**

- SQLite database storage for queue (extend existing `jobs` table)
- Leverages existing benchmark management UI from Alpha (left panel)
- New QueuePanel component in center panel (replace/enhance existing center content)
- Keyboard shortcuts: Q (queue selected), Space (toggle selection)

**Standalone:** ✅ Complete queueing functionality without needing execution capability
**Enables:** Epic 2 (execution), Epic 4 (monitoring)

---

### Epic 2: Sequential Queue Execution & Server-Side State Capture

**User Outcome:** Users can start queue processing, and the system reliably executes jobs one-at-a-time with guaranteed state capture on the remote server.

**Value Delivered:** Users can click "Start Queue" and trust that jobs will execute sequentially with 99.99% state reliability, even if their laptop loses connection or the app crashes. Jobs continue running on the remote server.

**FRs Covered:** FR155-FR157, FR159-FR163 + Architecture requirements (wrapper deployment, server DB schema, state files, reconciliation priority chain) (Total: 9 FRs + Architecture)

**Key Capabilities:**

- Start/pause/resume queue processing (Start Queue, Pause Queue, Resume Queue buttons)
- Sequential job execution (max_concurrent = 1 for Beta 1)
- Automatic progression to next job when current completes
- **Hybrid state capture solution (Research-driven decision):**
  - Bash wrapper script with `trap EXIT` (catches all exits except SIGKILL)
  - SQLite server database at `~/.solverpilot-server/server.db` (primary source of truth)
  - JSON state files at `~/.solverpilot-server/jobs/<job_id>.status` (fallback mechanism)
  - flock for atomic write guarantees (prevent race conditions)
- Wrapper deployment to remote server (embedded in Rust binary via `include_str!`)
- Server DB schema creation and management (auto-initialize on first use)
- Exit code capture and job status tracking (exit_code column in server DB)

**Architecture Notes:**

- Wrapper script (~50 lines bash) deployed to `~/.solverpilot/bin/job_wrapper.sh` on remote server
- Reconciliation priority: SQLite → State File → tmux check → Error
- **Data Migration:** ADD 3 columns to existing `jobs` table: `tmux_session_name TEXT UNIQUE`, `started_at TEXT`, `completed_at TEXT`
- NO schema rewrite - ALTER TABLE only (preserve Alpha data)
- New backend modules: `queue_service.rs`, `server_db.rs`, `wrapper.rs`
- New IPC commands: `start_queue`, `pause_queue`, `resume_queue`, `deploy_wrapper`, `init_server_db`

**Standalone:** ✅ Complete execution and state capture system - queue can execute and track jobs reliably
**Enables:** Epic 3 (reconciliation uses state capture), Epic 4 (monitoring uses state), Epic 5 (failure handling relies on state)

---

### Epic 3: Startup Reconciliation & "Walk Away" Confidence

**User Outcome:** Users can close their laptop, go to meetings, or even force-quit the app, then return hours or days later to see exactly what happened while they were away.

**Value Delivered:** Users gain "walk away confidence" - the core emotional promise of Beta 1. They see a Startup Resume Screen showing "3 completed while you were away, 1 failed (here's why), 5 pending." This is the make-or-break trust moment.

**FRs Covered:** FR159-FR163 + UX requirements (Startup Resume Screen, trust-building Tier 1 patterns) (Total: 5 FRs + UX)

**Key Capabilities:**

- **Startup reconciliation within 10 seconds:**
  - Query server SQLite database for job statuses (primary source)
  - Fallback to state files if DB unavailable or corrupted
  - Infer status from tmux session existence (`tmux has-session -t solverpilot_*`)
  - Mark as "state lost" if all sources fail (with clear error message)
- **Startup Resume Screen (NEW component - Tier 1 trust foundation):**
  - Visual summary: "3 completed • 1 failed • 5 pending"
  - Timestamps for completed jobs ("finished 2h ago")
  - Clear failure reasons with one-click access to logs
  - "Resume Queue" button to continue processing pending jobs
  - "Review Results" link to completed jobs
- **Crash recovery:**
  - Detect orphaned tmux sessions (session exists but no matching DB entry)
  - Match sessions to SQLite jobs by tmux_session_name
  - Resolve conflicts (crashed sessions, completed but not marked, running but finished)
- **Queue operation locking during reconciliation:**
  - Show progress indicator: "Syncing queue state... (3 seconds remaining)"
  - Block user from queuing/starting jobs until reconciliation completes
  - Queue user actions for execution after sync finishes

**UX Notes:**

- **Tier 1 trust foundation** - highest priority feature for Beta 1
- Make-or-break moment for user trust: "That's the moment where I either trust this tool or go back to my janky bash scripts" - Dr. Chen
- Clear, honest status (never show stale data as current)
- If reconciliation fails (server unreachable): "Cannot connect to server - last known state from [timestamp]"

**Architecture Notes:**

- New backend command: `reconcile_queue_state` (called on app startup)
- Reconciliation logic in `reconciliation.rs` module
- Priority chain implementation with fallback paths
- 5-10 second reconciliation window (configurable timeout)
- Lock mechanism prevents concurrent queue operations during sync

**Standalone:** ✅ Complete reconciliation and recovery system - users can trust state across sessions
**Enables:** Trust in system, Epic 4 (accurate monitoring depends on reconciliation), Epic 5 (failure recovery needs accurate state)

---

### Epic 4: Real-Time Monitoring & Progress Visibility

**User Outcome:** Users can instantly see queue status, monitor job progress in real-time, and review logs without clicking through multiple panels.

**Value Delivered:** Users have calm productivity - they can glance at the screen and immediately understand "3 running • 12 pending • 8 completed" without anxiety about whether work is progressing. No hunting for status.

**FRs Covered:** FR122-FR137 (queue monitoring subset) + UX requirements (always-visible status, glassmorphism, keyboard shortcuts) (Total: 16 FRs + UX)

**Key Capabilities:**

- **Always-visible queue status summary:**
  - Prominently displayed in queue panel header: "3 running • 12 pending • 8 completed"
  - Updates automatically every 2 seconds (backend polling)
  - Glanceable without navigation or clicking through panels
- **Real-time progress indicators:**
  - Elapsed time with live client-side counters ("Running for 3h 24m")
  - Progress parsing from logs `[45/100]` when solver outputs progress markers
  - NO fake ETAs (optimization problems unpredictable - honest progress only)
- **Live log streaming:**
  - 2-second polling for selected job only (not all jobs)
  - Last N lines with throttled updates (avoid render thrash from streaming)
  - Raw solver output in right panel (no prettified summaries that hide truth)
  - Auto-scroll to bottom for latest logs
- **Active job highlighting:**
  - Currently running job visually distinct from pending queue (prominent styling)
  - Visual hierarchy: Running (prominent) > Pending (subdued) > Completed (collapsed/grouped)
- **Queue position indicators:**
  - Show which job is #1, #2, #3 in pending queue
  - "Next to run" indicator for job at front of queue

**Architecture Notes:**

- 2-second backend polling for queue state (status changes only, not logs)
- Client-side JavaScript setInterval for elapsed time counters (no backend cost)
- Svelte $derived for computed properties (formatted elapsed time)
- 10-second lightweight health checks (separate from queue polling - echo command)
- New IPC command: `get_queue_summary` (returns counts and active job)

**UX Notes:**

- Glassmorphism with differentiated blur (2px panels, 12px header) for 60fps resize performance
- Panel opacity hierarchy (85%/75%/80%) for visual depth without multiple blur layers
- Keyboard shortcuts: Q (queue), R (retry), Space (toggle selection)
- Information density: py-2 spacing shows 12-15 jobs visible without scrolling
- Alternating row backgrounds for scanability of 50+ job queues

**Standalone:** ✅ Complete monitoring and visibility system - users can track queue in real-time
**Enables:** Informed decision-making, Epic 5 (identify failures to retry)

---

### Epic 5: Failed Job Handling & Queue Resilience

**User Outcome:** Users can handle job failures gracefully - failed jobs don't break the queue, error messages are clear, and retry is one-click away.

**Value Delivered:** Users feel accomplishment instead of frustration. They see "Job 3 failed (here's why), but Jobs 4-10 kept running" and can retry with confidence. Failed jobs prove system resilience rather than creating anxiety.

**FRs Covered:** FR184-FR191 + UX requirements (failed job indicators, one-click retry, raw error logs) (Total: 8 FRs + UX)

**Key Capabilities:**

- **Failed jobs don't block queue:**
  - Queue state machine: Failed job transitions to "failed" status → queue automatically starts next pending job
  - Concurrency slots freed immediately (if job 3 fails, job 4 starts immediately)
  - Queue processing never stops due to individual failure (resilience pattern)
- **Clear failure indicators:**
  - Red status badges for failed jobs (persist in queue, don't disappear or hide)
  - Error messages extracted from last 20 lines of log (auto-parsed)
  - Raw solver output accessible in logs panel (right panel - full context)
  - Failed jobs grouped for review (can filter queue to "show only failed")
- **One-click retry:**
  - `R` keyboard shortcut or "Retry" button
  - Returns job to pending status (re-queues at end)
  - Preserves original configuration (command-line arguments, environment variables)
  - Option to "Retry All Failed" for batch recovery
- **Auto-retry with backoff (optional, user-configurable):**
  - Configurable retry behavior: manual (default), auto with limits
  - Exponential backoff: 3 retries (immediate, 10s, 30s)
  - Quarantine repeatedly-failing jobs after 3 failures (prevent infinite loops)
  - Toast notification: "Job 'benchmark_01.py' failed 3 times - quarantined"
- **Failed job audit log:**
  - Track failure reasons for pattern analysis
  - Timestamp all failure events (failed_at, retry_count)
  - View failure history for specific benchmark
  - Export failure log to CSV for debugging

**Architecture Notes:**

- Queue state machine transitions: pending → running → completed|failed
- Failed status stored in SQLite with `exit_code` and `error_message`
- Retry creates new job entry with same benchmark_path and configuration (new job_id)
- New IPC commands: `retry_job`, `retry_all_failed`, `clear_completed_jobs`
- Error parsing regex for common solver failures (out of memory, timeout, invalid input)

**UX Notes:**

- Failed jobs clearly marked but queue continues (proves resilience, builds trust)
- No anxiety-inducing blocking behavior
- Failure is visible, acknowledged, and actionable (not hidden or minimized)
- Clear distinction: job failure vs system failure vs network failure

**Standalone:** ✅ Complete failure handling system - queue is production-ready and resilient
**Enables:** Production-ready reliability, user confidence to queue 50+ jobs

---

### Epic 6: Connection Resilience & Auto-Recovery

**User Outcome:** Users can trust that network disconnects won't break their work - the system automatically reconnects and transparently resumes operation.

**Value Delivered:** Users have "walk away confidence" even with unstable university VPNs or laptop sleep. They see "SSH connection lost 5 min ago, attempting reconnect..." and watch it transparently recover. Jobs continue on server regardless.

**FRs Covered:** Architecture requirements (bb8 connection pooling, health checks, auto-reconnect) + UX requirements (dual-channel connection indicators, reconnection messaging) (Total: Architecture + UX)

**Key Capabilities:**

- **Connection pooling (bb8) for ~10x performance improvement:**
  - Reuse existing SSH sessions (ControlMaster-style persistence)
  - Automatic connection lifecycle management
  - Pool configuration: max connections, idle timeout, connection timeout
  - Reduces SSH handshake overhead from ~500ms to <50ms for subsequent operations
- **Health checks every 10 seconds:**
  - Lightweight echo command (separate from queue polling - avoid mixing concerns)
  - Detect connection loss without blocking operations
  - Track connection status (connected, reconnecting, disconnected)
  - Update UI immediately when status changes
- **Auto-reconnect with retry logic:**
  - 3 retries with exponential backoff: immediate, 10s, 30s
  - Transparent state recovery after successful reconnection
  - Reconcile tmux state automatically (call reconciliation logic)
  - Toast notification: "Reconnecting to server... (attempt 2/3)"
- **Dual-channel connection status (UX trust pattern):**
  - **Ambient awareness:** Header bottom border color glow (green/yellow/red at 40% opacity)
  - **Active visibility:** Text indicator in queue panel header:
    - "● Connected" (green)
    - "⚠ Reconnecting..." (yellow with spinner)
    - "✗ Disconnected" (red)
  - Peripheral vision catches border glow, active users see explicit text status
- **Reconnection notifications:**
  - Toast: "SSH connection lost - attempting reconnect..."
  - Toast: "Reconnected - 2 jobs completed while disconnected" (shows reconciliation results)
  - Manual "Retry Connection" button if auto-reconnect fails (fallback to manual)
- **Queue behavior during disconnect:**
  - Pause queue state polling (don't show stale data as current)
  - Client-side elapsed time counters continue (no backend needed - proof of active execution)
  - Queue UI shows "Last updated: 30 seconds ago" (honesty about stale data)
  - On reconnection: Trigger reconciliation → update queue state → resume polling

**Architecture Notes:**

- bb8 pool integration with existing SSH manager from Alpha (`src-tauri/src/ssh/pool.rs`)
- ControlMaster socket management (reuse ~/.ssh/controlmasters/ directory)
- Health check separate from queue polling (avoid false positives from transient failures)
- New IPC commands: `get_connection_health`, `force_reconnect`
- Integration with reconciliation logic (Epic 3) for post-reconnect state sync

**UX Notes:**

- Transparency over false reassurance (honest problems better than fake calm)
- "I need to SEE that it's working even when disconnected" - Dr. Tanaka
- Peripheral awareness (border glow) + explicit status (text indicator) = dual-channel trust
- No silent failures - always inform user of connection issues with actionable messaging

**Standalone:** ✅ Complete connection resilience system - network issues don't break work
**Enables:** "Walk away" confidence with unstable networks, trust in system reliability

---

## FR Coverage Map

### Epic 1: Queue Foundation & Multi-Job Submission (20 FRs)

- **FR148:** User can select multiple benchmarks for queueing (shift-click, ctrl-click)
- **FR149:** User can add selected benchmarks to queue with one action
- **FR150:** System can store queue in SQLite database
- **FR151:** User can view all jobs in queue (pending, running, completed, failed)
- **FR152:** User can start queue processing → _Moved to Epic 2_
- **FR153:** User can pause queue (finish current job, stop starting new ones) → _Moved to Epic 2_
- **FR154:** User can resume paused queue → _Moved to Epic 2_
- **FR155:** System can process queue sequentially (one job at a time) → _Moved to Epic 2_
- **FR156:** System can automatically start next pending job when current completes → _Moved to Epic 2_
- **FR157:** User can cancel all pending jobs in queue
- **FR158:** User can remove specific job from queue (before execution)
- **FR164:** User can reorder jobs in queue (drag-and-drop or priority numbers)
- **FR165:** User can move job to front of queue
- **FR166:** User can move job to end of queue
- **FR167:** System can show queue position for each pending job
- **FR168:** System can estimate time remaining for queue (based on average job duration)
- **FR169:** User can filter queue view (show only pending, only failed, etc.)
- **FR173:** System can detect duplicate jobs in queue (same benchmark, same arguments)
- **FR174:** User can configure duplicate handling (allow, warn, prevent)
- **FR175:** System can warn when adding job that's already in queue
- **FR176:** User can replace existing queued job with new configuration

### Epic 2: Sequential Queue Execution & State Capture (9 FRs + Architecture)

- **FR152:** User can start queue processing
- **FR153:** User can pause queue (finish current job, stop starting new ones)
- **FR154:** User can resume paused queue
- **FR155:** System can process queue sequentially (one job at a time)
- **FR156:** System can automatically start next pending job when current completes
- **FR159:** System can persist queue across application restarts → _Shared with Epic 3_
- **FR160:** User can resume queue processing after reopening application → _Shared with Epic 3_
- **FR161:** System can recover from SolverPilot crashes without losing queue state → _Shared with Epic 3_
- **FR162:** System can detect partially-completed queues on startup → _Shared with Epic 3_

**Architecture Requirements:**

- Bash wrapper script deployment with `trap EXIT`
- SQLite server database at `~/.solverpilot-server/server.db`
- JSON state files at `~/.solverpilot-server/jobs/<job_id>.status`
- flock for atomic write guarantees
- Reconciliation priority chain: SQLite → State File → tmux → Error
- ALTER TABLE jobs: ADD `tmux_session_name TEXT UNIQUE`, `started_at TEXT`, `completed_at TEXT`
- New backend modules: `queue_service.rs`, `server_db.rs`, `wrapper.rs`
- Wrapper embedded in Rust binary: `include_str!("../scripts/job_wrapper.sh")`

### Epic 3: Startup Reconciliation & "Walk Away" Confidence (5 FRs + UX)

- **FR159:** System can persist queue across application restarts
- **FR160:** User can resume queue processing after reopening application
- **FR161:** System can recover from SolverPilot crashes without losing queue state
- **FR162:** System can detect partially-completed queues on startup
- **FR163:** User can view queue recovery status with clear indicators

**UX Requirements:**

- Startup Resume Screen (NEW component - Tier 1 trust foundation)
- Visual summary: "3 completed • 1 failed • 5 pending"
- Timestamps and clear failure reasons
- "Resume Queue" button
- Queue operation locking during reconciliation (10-second window)
- Progress indicator: "Syncing queue state... (3 seconds remaining)"
- Honest status messaging (never show stale data as current)

**Architecture Requirements:**

- `reconcile_queue_state` IPC command
- `reconciliation.rs` module with priority chain logic
- Orphaned tmux session detection
- Conflict resolution (crashed sessions, completed but unmarked)

### Epic 4: Real-Time Monitoring & Progress Visibility (16 FRs + UX)

- **FR122:** User can view live log output from running job
- **FR123:** System can stream logs from remote tmux session
- **FR124:** System can parse progress indicators (e.g., [x/y] patterns)
- **FR125:** System can display elapsed time with live counter
- **FR126:** System can detect job completion (exit patterns)
- **FR127:** System can show final job status (success, failure, cancelled)
- **FR128:** User can view job status when network is offline (from local database)
- **FR129:** System can reconnect to running job after network interruption → _Shared with Epic 6_
- **FR130:** User can check status of job started in previous session → _Covered by Epic 3 reconciliation_
- **FR131:** User can view complete log history for finished jobs
- **FR132:** User can export logs to text file
- **FR133:** System can preserve logs in database for historical analysis
- **FR134:** User can see which job is currently running across all projects
- **FR135:** System can detect job timeout (configurable max execution time)
- **FR136:** System can warn about abnormally long-running jobs
- **FR137:** User can receive notifications for job state changes

**UX Requirements:**

- Always-visible queue status summary: "3 running • 12 pending • 8 completed"
- 2-second automatic updates (backend polling)
- Glanceable status without navigation
- Real-time progress indicators (elapsed time, [x/y] parsing)
- NO fake ETAs (honest progress only)
- Live log streaming with throttled updates
- Active job highlighting (visual hierarchy)
- Glassmorphism (2px panels, 12px header) for 60fps resize
- Keyboard shortcuts: Q (queue), R (retry), Space (toggle)

**Architecture Requirements:**

- 2-second backend polling for queue state (status changes only)
- Client-side JavaScript setInterval for elapsed time counters
- Svelte $derived for computed properties
- 10-second lightweight health checks (echo command, separate from polling)
- `get_queue_summary` IPC command

### Epic 5: Failed Job Handling & Queue Resilience (8 FRs + UX)

- **FR184:** System can prevent cascade failures (failed job doesn't stop queue)
- **FR185:** User can configure retry behavior for failed jobs
- **FR186:** System can automatically retry failed jobs with exponential backoff
- **FR187:** System can auto-reconnect SSH after transient connection failures → _Shared with Epic 6_
- **FR188:** System can clean up orphaned tmux sessions from previous crashes → _Shared with Epic 3_
- **FR189:** System can detect server reboots and re-establish connections → _Shared with Epic 6_
- **FR190:** User can view failure reasons for all failed jobs in queue
- **FR191:** System can quarantine repeatedly-failing jobs (prevent infinite retries)

**UX Requirements:**

- Failed jobs don't block queue (state machine: failed → next pending)
- Clear failure indicators: red status badges, persist (don't disappear)
- Error messages extracted from last 20 lines of log
- Raw solver output accessible in logs panel
- One-click retry: `R` keyboard shortcut or "Retry" button
- "Retry All Failed" for batch recovery
- Queue continues executing (proves resilience, builds trust)

**Architecture Requirements:**

- Queue state machine: pending → running → completed|failed
- Failed status in SQLite: `exit_code`, `error_message`
- Retry creates new job entry (new job_id, same benchmark_path)
- `retry_job`, `retry_all_failed`, `clear_completed_jobs` IPC commands
- Error parsing regex for common solver failures

### Epic 6: Connection Resilience & Auto-Recovery (Architecture + UX)

- **FR129:** System can reconnect to running job after network interruption
- **FR187:** System can auto-reconnect SSH after transient connection failures
- **FR189:** System can detect server reboots and re-establish connections

**Architecture Requirements:**

- bb8 connection pooling (~10x performance improvement)
- Reuse SSH sessions (ControlMaster-style persistence)
- Pool configuration: max connections, idle timeout, connection timeout
- 10-second health checks (lightweight echo command)
- 3 retries with exponential backoff: immediate, 10s, 30s
- Transparent state recovery after reconnection
- Automatic reconciliation trigger on successful reconnect
- `get_connection_health`, `force_reconnect` IPC commands
- Integration with existing SSH manager (`src-tauri/src/ssh/pool.rs`)

**UX Requirements:**

- Dual-channel connection status:
  - Ambient: Header bottom border glow (green/yellow/red at 40% opacity)
  - Active: Text indicator ("● Connected", "⚠ Reconnecting...", "✗ Disconnected")
- Reconnection toast notifications with reconciliation results
- Manual "Retry Connection" button (fallback if auto-reconnect fails)
- Queue pauses polling during disconnect (don't show stale data)
- Client-side elapsed timers continue (proof of active execution)
- "Last updated: 30 seconds ago" indicator (honesty about stale data)
- Transparency over false reassurance (honest problems better than fake calm)

---

## Coverage Summary

**Total FRs Addressed in Beta 1:** 54 Queue Management FRs (FR148-FR191, with some shared across epics)

**Additional Requirements Covered:**

- Architecture decisions: Hybrid state capture (bash wrapper + SQLite + state files), reconciliation priority chain, bb8 pooling
- UX requirements: WCAG AAA, keyboard shortcuts, glassmorphism, dual-channel indicators, Startup Resume Screen
- Research decisions: Wrapper deployment pattern, server DB schema, failure modes & edge cases

**FR Distribution Across Epics:**

- Epic 1: 20 FRs (queue foundation, organization)
- Epic 2: 9 FRs + Architecture (execution, state capture)
- Epic 3: 5 FRs + UX (reconciliation, trust)
- Epic 4: 16 FRs + UX (monitoring, visibility)
- Epic 5: 8 FRs + UX (failure handling, resilience)
- Epic 6: 3 FRs + Architecture + UX (connection resilience)

**Note:** Some FRs are shared across multiple epics due to interconnected nature (e.g., FR159-FR163 persistence/recovery spans Epic 2 and Epic 3, FR187/FR189 connection handling spans Epic 5 and Epic 6).

**Alpha FRs (Already Implemented):** FR1-FR147 covering Configuration, Projects, Benchmarks, Dependencies, Single Job Execution, and Single Job Monitoring are preserved and NOT re-implemented in Beta 1.

---

# Epic Stories

## Epic 1: Queue Foundation & Multi-Job Submission

**Epic Goal:** Users can select multiple benchmarks, add them to a queue, organize the queue, and view all queued jobs with their statuses.

**Value Delivered:** Users can organize their benchmark work into a unified queue instead of running jobs one-at-a-time. They can batch 10, 20, or 50 benchmarks before lunch and prepare for execution.

**FRs Covered:** FR148-FR158, FR164-FR169, FR173-FR176 (Total: 20 FRs)

---

### Story 1.1: Multi-Select Benchmarks in Left Panel

As a researcher,
I want to select multiple benchmarks using keyboard and mouse interactions (shift-click, ctrl-click, keyboard shortcuts),
So that I can efficiently queue many benchmarks for batch execution without clicking "Add to Queue" repeatedly.

**Acceptance Criteria:**

**Given** I have a project with 20+ benchmarks loaded in the left panel
**When** I click on benchmark_01.py
**Then** the benchmark is highlighted with selected state (visual indicator)

**Given** I have benchmark_01.py selected
**When** I hold Shift and click benchmark_05.py
**Then** benchmarks 01, 02, 03, 04, and 05 are all selected (range selection)

**Given** I have benchmarks 01-05 selected
**When** I hold Ctrl/Cmd and click benchmark_10.py
**Then** benchmark_10.py is added to the selection without deselecting 01-05 (individual toggle)

**Given** I have multiple benchmarks selected
**When** I press the Space key on a focused benchmark
**Then** that benchmark's selection state toggles (selected ↔ unselected)

**Given** I have benchmarks selected
**When** I press the Q key
**Then** a "Queue Selected" action is triggered (preparation for Story 1.2)

**Given** I have benchmarks selected
**When** I click elsewhere or press Escape
**Then** all selections are cleared

**And** visual feedback shows selected count: "3 benchmarks selected" in panel header
**And** selected benchmarks have distinct styling (background color, border, checkmark icon)
**And** keyboard focus is visible with focus ring for accessibility (WCAG AAA)
**And** multi-select works identically on Windows, macOS, and Linux (platform consistency)

**Technical Notes:**

- Enhance existing BenchmarkList component from Alpha (`src/lib/features/benchmarks/`)
- Use Svelte 5 runes: `let selectedBenchmarks = $state<string[]>([])` (NOT legacy stores)
- Keyboard event handlers for Space, Q, Escape
- Mouse event handlers with shiftKey/ctrlKey/metaKey detection
- NO backend changes required (frontend-only story)

**FRs Fulfilled:** FR148 (multi-select), FR149 (add to queue action - UI trigger only)

---

### Story 1.2: Queue Storage in SQLite Database

As a researcher,
I want the system to persist my queue in the database,
So that queued jobs survive application restarts and I can resume my work later.

**Acceptance Criteria:**

**Given** the local SQLite database exists at `~/.solverpilot/local.db`
**When** the application starts for the first time with this story
**Then** the `jobs` table is altered to support queue functionality with new columns:

- `queue_position INTEGER` (NULL for non-queued jobs, 1-N for queued jobs)
- `queued_at TEXT` (ISO 8601 timestamp when job was added to queue)

**Given** the database schema has been migrated
**When** I verify the migration
**Then** existing Alpha job data is preserved (no data loss)
**And** the migration completes in <2 seconds

**Given** I have 3 benchmarks selected in the left panel
**When** I press Q or click "Queue Selected" button
**Then** 3 new job entries are inserted into the `jobs` table with:

- `status = 'pending'`
- `queue_position` assigned sequentially (1, 2, 3)
- `queued_at` set to current timestamp
- `benchmark_name` from selected benchmarks
- `project_id` from current active project

**Given** I have 5 jobs already queued (queue_position 1-5)
**When** I queue 2 more benchmarks
**Then** the new jobs get queue_position 6 and 7 (append to end)

**Given** I restart the application
**When** I query the database
**Then** all queued jobs are still present with correct queue_position and status

**And** SQL queries use parameterization to prevent SQL injection (NFR-S10)
**And** database operations complete atomically with ACID properties (NFR-R7)
**And** all database errors return Result<T, String> with descriptive messages (no unwrap/expect - clippy enforced)

**Technical Notes:**

- Create new Tauri IPC command: `queue_benchmarks(benchmark_ids: Vec<i64>) -> Result<Vec<Job>, String>`
- ALTER TABLE jobs ADD COLUMN queue_position INTEGER, queued_at TEXT
- Use SQLx compile-time checked queries: `sqlx::query!("INSERT INTO jobs ...")`
- Backend module: extend `src-tauri/src/commands.rs` and `src-tauri/src/db.rs`
- Frontend API: add `queueBenchmarks(ids: number[])` to `src/lib/api.ts`

**FRs Fulfilled:** FR149 (add to queue), FR150 (SQLite storage), FR159 (persistence)

---

### Story 1.3: Queue Panel UI - View Queued Jobs

As a researcher,
I want to see all queued jobs in a dedicated center panel with their statuses,
So that I can monitor my queue at a glance and know what's pending, running, completed, or failed.

**Acceptance Criteria:**

**Given** I have 10 jobs queued in the database
**When** I open the application
**Then** the center panel displays a "Queue" panel showing all 10 jobs

**Given** the Queue panel is visible
**When** I look at a queued job entry
**Then** I see:

- Benchmark name (e.g., "benchmark_01.py")
- Status badge (Pending/Running/Completed/Failed with color coding)
- Queue position number for pending jobs ("#1", "#2", "#3")
- Timestamp (queued time for pending, elapsed time for running, completed time for finished)

**Given** I have a mix of job statuses (3 pending, 2 running, 5 completed)
**When** the Queue panel renders
**Then** jobs are grouped by status with visual hierarchy:

- Running jobs: Prominent styling (bold, larger, at top)
- Pending jobs: Subdued styling (normal weight, middle section)
- Completed jobs: Collapsed/grouped styling (subtle, bottom section)

**Given** the Queue panel displays jobs
**When** the backend polling updates job statuses (every 2 seconds - Epic 4)
**Then** the UI reactively updates without full page refresh (Svelte $state reactivity)

**Given** I have 50+ jobs in the queue
**When** I scroll the Queue panel
**Then** the list scrolls smoothly with py-2 spacing showing 12-15 jobs visible at 1080p-1440p screens
**And** alternating row backgrounds (even:bg-white/2) improve scanability

**Given** I have no jobs queued
**When** the Queue panel loads
**Then** I see an empty state message: "No jobs in queue. Select benchmarks and press Q to get started."

**And** status badges use triple encoding (color + icon + text) for WCAG AAA accessibility
**And** the panel uses glassmorphism styling (bg-slate-900/75 opacity, 2px backdrop-blur)
**And** the panel is resizable with minimum width 400px

**Technical Notes:**

- Create NEW component: `src/lib/features/queue/QueuePanel.svelte`
- Replace/enhance center panel in MainLayout (`src/lib/layout/MainLayout.svelte`)
- Use Svelte 5 runes for reactive state: `let jobs = $state<Job[]>([])`
- Create IPC command: `get_all_queue_jobs() -> Result<Vec<Job>, String>`
- Status badge colors (TailwindCSS oklch):
  - Pending: blue-500
  - Running: green-500
  - Completed: gray-400
  - Failed: red-500
- Queue position indicator: `<span class="text-xs text-gray-400">#{job.queue_position}</span>`

**FRs Fulfilled:** FR151 (view all jobs in queue), FR167 (queue position indicators)

---

### Story 1.4: Queue Job Management - Remove & Reorder

As a researcher,
I want to remove specific jobs from the queue and reorder them,
So that I can adjust my queue dynamically as priorities change without starting over.

**Acceptance Criteria:**

**Given** I have 10 jobs in the queue with positions 1-10
**When** I click the "Remove" button (trash icon) on job #5
**Then** job #5 is deleted from the queue
**And** jobs #6-10 are renumbered to positions #5-9 (queue positions shift down)
**And** the UI updates immediately to reflect the change

**Given** I have pending jobs in the queue
**When** I select a job and press the Delete key
**Then** the job is removed from the queue (keyboard accessibility)

**Given** I have job #5 selected
**When** I click "Move to Front" button or action
**Then** job #5 becomes position #1
**And** previous jobs #1-4 shift to positions #2-5

**Given** I have job #3 selected
**When** I click "Move to End" button
**Then** job #3 moves to the last position in pending queue
**And** jobs #4+ shift up by one position

**Given** I have job #5 in the queue
**When** I drag job #5 and drop it at position #2
**Then** job #5 is reordered to position #2
**And** jobs #2-4 shift to positions #3-5 (make room)

**Given** I have jobs with status = 'running' or 'completed'
**When** I attempt to remove or reorder them
**Then** the action is blocked with a toast notification: "Cannot modify jobs that are running or completed"

**Given** I click "Cancel All Pending" button in Queue panel header
**When** I confirm the action
**Then** all jobs with status = 'pending' are deleted from the queue
**And** running/completed jobs remain untouched

**And** all queue modifications persist to the database immediately
**And** drag-and-drop provides visual feedback (ghost element, drop zones highlighted)
**And** undo is NOT required (user confirms destructive actions via modal)

**Technical Notes:**

- Create Tauri IPC commands:
  - `remove_job_from_queue(job_id: i64) -> Result<(), String>`
  - `reorder_queue_job(job_id: i64, new_position: i32) -> Result<(), String>`
  - `move_job_to_front(job_id: i64) -> Result<(), String>`
  - `move_job_to_end(job_id: i64) -> Result<(), String>`
  - `cancel_all_pending_jobs() -> Result<u32, String>` (returns count deleted)
- Backend logic: recalculate queue_position for all affected jobs after removal/reorder
- Frontend: use `@sveltejs/sortable` or native HTML5 drag-and-drop API
- Confirmation modal for destructive actions (Cancel All)

**FRs Fulfilled:** FR157 (cancel all pending), FR158 (remove specific job), FR164 (reorder), FR165 (move to front), FR166 (move to end)

---

### Story 1.5: Duplicate Detection & Queue Filtering

As a researcher,
I want the system to detect duplicate jobs and filter the queue view by status,
So that I avoid accidentally queuing the same benchmark twice and can focus on specific job states.

**Acceptance Criteria:**

**Given** I have benchmark_01.py already queued with status = 'pending'
**When** I select benchmark_01.py again and press Q to queue it
**Then** a toast notification appears: "benchmark_01.py is already in the queue (pending). Add anyway?"
**And** I see two options: "Add Anyway" (duplicate allowed) or "Cancel"

**Given** the duplicate detection toast is shown
**When** I click "Add Anyway"
**Then** a second instance of benchmark_01.py is added to the queue with a new job_id
**And** both jobs are visible in the Queue panel

**Given** the duplicate detection toast is shown
**When** I click "Cancel"
**Then** no new job is created
**And** the toast dismisses

**Given** I am in application settings
**When** I configure duplicate handling to "Prevent" (vs default "Warn")
**Then** attempting to queue a duplicate shows error toast: "benchmark_01.py is already queued. Duplicates are not allowed."
**And** no "Add Anyway" option is presented

**Given** I have a queue with 20 jobs (5 pending, 3 running, 10 completed, 2 failed)
**When** I click the "Filter" dropdown in Queue panel header
**Then** I see filter options: "All", "Pending", "Running", "Completed", "Failed"

**Given** I select filter: "Pending"
**When** the filter is applied
**Then** only jobs with status = 'pending' are visible in the Queue panel
**And** the panel header shows: "Queue (5 pending)" to indicate active filter

**Given** I select filter: "Failed"
**When** the filter is applied
**Then** only failed jobs are visible
**And** each failed job shows its error message snippet

**Given** I have a filter active (e.g., "Pending")
**When** I select "All" filter
**Then** all jobs are visible again regardless of status

**And** duplicate detection compares benchmark_name AND status (only warn if status = 'pending' or 'running')
**And** completed/failed jobs do NOT trigger duplicate warnings (user may want to retry)
**And** filter state persists to localStorage (user preference remembered across sessions)

**Technical Notes:**

- Extend `queue_benchmarks` command with duplicate check logic
- Query database: `SELECT COUNT(*) FROM jobs WHERE benchmark_name = ? AND status IN ('pending', 'running')`
- Add user setting: `duplicate_handling: 'warn' | 'prevent' | 'allow'` in config.toml
- Frontend: filter dropdown in QueuePanel header
- Use Svelte $derived for filtered job list: `let filteredJobs = $derived(jobs.filter(...))`
- Toast notifications via existing toast store (`src/lib/stores/toast.svelte.ts`)

**FRs Fulfilled:** FR169 (filter queue view), FR173 (duplicate detection), FR174 (duplicate handling config), FR175 (duplicate warning), FR176 (replace existing job - via "Add Anyway")

---

## Epic 1 Summary

**Stories Created:** 5 stories
**Total FRs Covered:** 20 FRs (FR148-FR158, FR164-FR169, FR173-FR176)

**Story Breakdown:**

- Story 1.1: Multi-select UI (FR148, FR149 trigger)
- Story 1.2: Queue storage backend (FR149, FR150, FR159)
- Story 1.3: Queue panel display (FR151, FR167)
- Story 1.4: Remove & reorder (FR157, FR158, FR164-FR166)
- Story 1.5: Duplicates & filtering (FR169, FR173-FR176)

**Implementation Order:** Sequential (1.1 → 1.2 → 1.3 → 1.4 → 1.5)
**Dependencies:** Each story builds on previous, no forward dependencies

**Ready for Development:** ✅ All stories have clear acceptance criteria, technical notes, and FR mappings

---

## Epic 2: Sequential Queue Execution & Server-Side State Capture

**Epic Goal:** Users can start queue processing, and the system reliably executes jobs one-at-a-time with guaranteed state capture on the remote server.

**Value Delivered:** Users can click "Start Queue" and trust that jobs will execute sequentially with 99.99% state reliability, even if their laptop loses connection or the app crashes. Jobs continue running on the remote server.

**FRs Covered:** FR155-FR157, FR159-FR163 + Architecture requirements (wrapper deployment, server DB schema, state files, reconciliation priority chain) (Total: 9 FRs + Architecture)

---

### Story 2.1: Bash Wrapper Script - State Capture Foundation

As a system architect,
I want a bash wrapper script that captures job state using trap EXIT and writes to both SQLite and state files,
So that job completion state is guaranteed with 99.99% reliability even if the wrapper is killed.

**Acceptance Criteria:**

**Given** the wrapper script exists at `src-tauri/scripts/job_wrapper.sh`
**When** I review the script structure
**Then** it contains:

- Shebang: `#!/bin/bash`
- Strict mode: `set -euo pipefail`
- Trap EXIT handler: `trap cleanup EXIT`
- Cleanup function that captures `$?` exit code
- Double-write logic: SQLite + JSON state file

**Given** the wrapper script is executed with job_id and command arguments
**When** the job starts
**Then** the wrapper:

- Creates directories: `~/.solverpilot-server/{jobs,locks}`
- Acquires exclusive lock: `exec 200>"$LOCK_FILE"; flock -x 200`
- Writes "running" status to SQLite: `UPDATE jobs SET status='running', started_at=datetime('now') WHERE id='$JOB_ID'`
- Writes "running" status to state file: `~/.solverpilot-server/jobs/$JOB_ID.status` (JSON format)
- Executes the actual job command: `"$@"`

**Given** the job completes successfully with exit code 0
**When** the trap EXIT handler runs
**Then** the cleanup function:

- Captures exit_code: `local exit_code=$?`
- Sets status to "completed"
- Writes to SQLite: `UPDATE jobs SET status='completed', completed_at=datetime('now'), exit_code=0 WHERE id='$JOB_ID'`
- Writes to state file with JSON structure:

```json
{
  "id": "$JOB_ID",
  "status": "completed",
  "exit_code": 0,
  "completed_at": "2026-01-11T14:23:45Z",
  "user": "$USER"
}
```

- Releases lock: `flock -u 200`

**Given** the job fails with exit code 1
**When** the trap EXIT handler runs
**Then** status is set to "failed" (not "completed")
**And** exit_code is captured correctly: `exit_code=1`

**Given** the SQLite write fails (e.g., database locked)
**When** the cleanup function executes
**Then** the wrapper gracefully falls back to state file only
**And** logs the error to stderr: `echo "WARNING: Failed to update SQLite, state file written" >&2`

**Given** the wrapper is killed with SIGTERM or SIGINT
**When** the signal is received
**Then** the trap EXIT handler still runs (captures exit)

**Given** the wrapper is killed with SIGKILL (kill -9)
**When** the process is terminated
**Then** trap EXIT does NOT run (expected limitation - documented in comments)
**And** reconciliation logic (Epic 3) will detect missing state

**And** the script is POSIX-compliant bash (works on Ubuntu, Debian, RHEL, Alpine)
**And** the script is ~50 lines total (simple, minimal bug surface)
**And** flock ensures atomic writes (no race conditions with concurrent jobs)
**And** all timestamps use ISO 8601 format: `date -Iseconds`

**Technical Notes:**

- Script location: `src-tauri/scripts/job_wrapper.sh`
- Will be embedded in Rust binary via `include_str!("../scripts/job_wrapper.sh")` (Story 2.3)
- Test wrapper locally: `./job_wrapper.sh test-job-123 python3 test_script.py`
- Verify trap EXIT: `./job_wrapper.sh test exit 0` (should capture exit 0)
- Verify SIGKILL limitation: `kill -9 <pid>` (trap won't run, state file won't be written)

**FRs Fulfilled:** Architecture requirement (bash wrapper with trap EXIT)

---

### Story 2.2: Server-Side SQLite Database Schema & Initialization

As a system architect,
I want a server-side SQLite database at `~/.solverpilot-server/server.db` with the jobs table schema,
So that the remote server can store job state independently of the client.

**Acceptance Criteria:**

**Given** the server database does not exist
**When** the initialization command runs on the remote server
**Then** a new SQLite database is created at `~/.solverpilot-server/server.db`
**And** the database file has permissions 0600 (user read/write only - security)

**Given** the database is being initialized
**When** the jobs table is created
**Then** the schema matches this exact structure:

```sql
CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    user TEXT NOT NULL DEFAULT 'default',
    benchmark_path TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('queued', 'running', 'completed', 'failed', 'killed')),
    tmux_session_name TEXT UNIQUE,
    queued_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    exit_code INTEGER,
    error_message TEXT,
    log_file TEXT,
    progress_current INTEGER,
    progress_total INTEGER
);
```

**Given** the jobs table is created
**When** I verify the indexes
**Then** the following indexes exist for query performance:

- `CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);`
- `CREATE INDEX IF NOT EXISTS idx_jobs_user ON jobs(user);`
- `CREATE INDEX IF NOT EXISTS idx_jobs_queued_at ON jobs(queued_at);`

**Given** multiple jobs are being inserted concurrently
**When** the database is in WAL mode
**Then** concurrent writes do not block reads excessively (WAL benefits)

**Given** the database initialization script is idempotent
**When** I run the initialization twice
**Then** no errors occur (CREATE TABLE IF NOT EXISTS)
**And** existing data is preserved

**Given** the wrapper script writes to the server database
**When** SQLite is unavailable (e.g., database locked, disk full)
**Then** the write operation fails gracefully
**And** the wrapper logs the error but continues (state file fallback)

**And** the database is created with WAL mode enabled: `PRAGMA journal_mode=WAL;`
**And** busy timeout is set to 5000ms: `PRAGMA busy_timeout=5000;`
**And** foreign keys are enabled: `PRAGMA foreign_keys=ON;` (future-proofing)

**Technical Notes:**

- Create Rust module: `src-tauri/src/server_db.rs` with initialization logic
- SQL schema stored as constant: `const SCHEMA: &str = include_str!("../sql/server_schema.sql");`
- New Tauri command: `init_server_db() -> Result<(), String>` (executed via SSH)
- SSH command to initialize: `ssh user@host "sqlite3 ~/.solverpilot-server/server.db < init.sql"`
- Test locally: Create server.db in temp directory, verify schema with `sqlite3 server.db ".schema jobs"`

**FRs Fulfilled:** Architecture requirement (SQLite server database schema)

---

### Story 2.3: Wrapper Deployment via SSH

As a system operator,
I want the wrapper script automatically deployed to the remote server on first queue execution,
So that the state capture infrastructure is installed without manual setup.

**Acceptance Criteria:**

**Given** the wrapper script exists at `src-tauri/scripts/job_wrapper.sh`
**When** the Rust backend compiles
**Then** the wrapper is embedded in the binary via `include_str!("../scripts/job_wrapper.sh")`
**And** the wrapper is accessible as a static string at runtime

**Given** the application starts and connects to the remote server
**When** I check for wrapper installation
**Then** a new Tauri command `check_wrapper_installed() -> Result<bool, String>` queries via SSH:

- `ssh user@host "test -f ~/.solverpilot/bin/job_wrapper.sh && echo 'installed' || echo 'missing'"`

**Given** the wrapper is NOT installed on the remote server
**When** the first queue execution attempt occurs
**Then** the system automatically calls `deploy_wrapper()` command
**And** a toast notification shows: "Installing queue infrastructure on server..."

**Given** the deploy_wrapper command executes
**When** deployment proceeds
**Then** the following steps occur in sequence:

1. Create remote directory: `ssh user@host "mkdir -p ~/.solverpilot/bin"`
2. Write wrapper content via heredoc:

```bash
ssh user@host "cat > ~/.solverpilot/bin/job_wrapper.sh << 'WRAPPER_EOF'
<embedded wrapper script content>
WRAPPER_EOF"
```

3. Make wrapper executable: `ssh user@host "chmod +x ~/.solverpilot/bin/job_wrapper.sh"`
4. Initialize server database: call `init_server_db()` from Story 2.2

**Given** the deployment succeeds
**When** I verify installation
**Then** `check_wrapper_installed()` returns `true`
**And** a toast notification shows: "Queue infrastructure ready ✓"

**Given** the deployment fails (e.g., SSH connection lost, permission denied)
**When** the error occurs
**Then** the operation returns descriptive error: `Result::Err("Failed to deploy wrapper: permission denied on remote ~/.solverpilot/bin")`
**And** the user sees error toast: "Failed to install queue infrastructure. Check server permissions."
**And** queue execution is blocked until deployment succeeds

**Given** the wrapper is already installed
**When** I attempt to deploy again
**Then** the system detects existing installation
**And** deployment is skipped (idempotent - no unnecessary writes)

**And** wrapper deployment uses bb8 connection pool for SSH efficiency (reuse existing connection)
**And** wrapper version is tracked in database for debugging: `INSERT INTO metadata (key, value) VALUES ('wrapper_version', '1.0.0')`
**And** deployment completes in <5 seconds on 50ms latency connection

**Technical Notes:**

- Create Rust module: `src-tauri/src/wrapper.rs`
- New Tauri commands:
  - `check_wrapper_installed() -> Result<bool, String>`
  - `deploy_wrapper() -> Result<(), String>`
- Embed wrapper: `const WRAPPER_SCRIPT: &str = include_str!("../scripts/job_wrapper.sh");`
- SSH write via heredoc to avoid escaping issues
- Call deployment automatically before first queue execution
- Frontend: show deployment progress in toast notification

**FRs Fulfilled:** Architecture requirement (wrapper deployment pattern)

---

### Story 2.4: Queue Execution Backend - Sequential Job Processing

As a researcher,
I want the backend to process queued jobs sequentially (one at a time),
So that jobs execute in order without overloading the remote server.

**Acceptance Criteria:**

**Given** I have 5 jobs in the queue with status = 'pending'
**When** the queue execution starts
**Then** the backend selects the job with the lowest queue_position (job #1)
**And** only ONE job is executed at a time (max_concurrent = 1 for Beta 1)

**Given** job #1 is selected for execution
**When** the execution process begins
**Then** the following steps occur in sequence:

1. Update local database: `UPDATE jobs SET status='running', started_at=datetime('now') WHERE id=?`
2. Rsync project files to remote server: `rsync -avz --delete <local_project_path> user@host:<remote_base_dir>/`
3. Create tmux session name: `solverpilot_${USER}_${JOB_ID:0:8}` (unique, collision-resistant)
4. Start job in tmux with wrapper:

```bash
tmux new-session -d -s <session_name> \
  "~/.solverpilot/bin/job_wrapper.sh <job_id> python3 <benchmark_path>"
```

5. Return immediately (non-blocking - job continues on server)

**Given** job #1 is now running on the remote server
**When** I query the local database
**Then** the job status is 'running'
**And** the tmux_session_name is stored in the local database

**Given** job #1 completes (wrapper writes to server DB)
**When** the backend polls for job status (2-second interval - Epic 4)
**Then** the backend detects completion by querying server DB via SSH:

- `ssh user@host "sqlite3 ~/.solverpilot-server/server.db 'SELECT status, exit_code FROM jobs WHERE id=<job_id>'"`

**Given** the backend detects job #1 is completed
**When** the status update occurs
**Then** the local database is updated:

- `UPDATE jobs SET status='completed', completed_at=<remote_completed_at>, exit_code=<exit_code> WHERE id=?`
  **And** the backend immediately selects the next pending job (job #2) and starts execution

**Given** I have jobs #1, #2, #3 pending
**When** job #1 completes
**Then** job #2 starts automatically (no user intervention required)
**And** when job #2 completes, job #3 starts automatically (continuous processing)

**Given** the queue is empty (no pending jobs)
**When** the backend checks for next job
**Then** the execution loop stops gracefully
**And** a toast notification shows: "Queue completed - all jobs finished"

**Given** the rsync operation fails (network error, permission denied)
**When** the failure is detected
**Then** the job status is set to 'failed'
**And** error_message is populated: "Failed to sync project files: <error_details>"
**And** the queue continues to the next job (failed jobs don't block queue - Epic 5)

**And** all SSH operations use bb8 connection pool for performance
**And** tmux session creation includes error handling (check if session name conflicts)
**And** all database operations return Result<T, String> (no unwrap/expect - clippy enforced)

**Technical Notes:**

- Create Rust module: `src-tauri/src/queue_service.rs` with execution loop
- New Tauri command: `start_queue_processing() -> Result<(), String>`
- Execution loop runs in background Tokio task (non-blocking)
- Use tokio::time::interval for 2-second polling
- Query server DB via SSH: `ssh user@host "sqlite3 ~/.solverpilot-server/server.db 'SELECT ...'"`
- Rsync command: `rsync -avz --delete --exclude '.git' --exclude '__pycache__'`
- Tmux session naming: ensure uniqueness with timestamp suffix if collision detected

**FRs Fulfilled:** FR155 (sequential execution), FR156 (auto-start next job), FR159 (persistence - jobs continue)

---

### Story 2.5: Start/Pause/Resume Queue Controls (Frontend + Backend)

As a researcher,
I want to start, pause, and resume queue processing from the UI,
So that I can control when jobs execute and temporarily stop the queue without losing progress.

**Acceptance Criteria:**

**Given** I have 10 pending jobs in the queue
**When** I click the "Start Queue" button in the Queue panel header
**Then** the button triggers `start_queue_processing()` backend command
**And** the button changes to "Pause Queue" (toggle state)
**And** the first pending job immediately begins execution

**Given** the queue is processing (3 jobs running)
**When** I click the "Pause Queue" button
**Then** the backend stops selecting new jobs after current job completes
**And** currently running jobs continue to completion (graceful pause)
**And** the button changes to "Resume Queue"

**Given** job #1 is running and I paused the queue
**When** job #1 completes
**Then** job #2 does NOT start automatically
**And** the queue remains paused
**And** a toast notification shows: "Queue paused - 9 jobs remaining"

**Given** the queue is paused with 9 pending jobs
**When** I click "Resume Queue" button
**Then** the backend resumes execution loop
**And** the next pending job (job #2) starts immediately
**And** the button changes back to "Pause Queue"

**Given** the queue is empty (no pending jobs)
**When** I look at the Queue panel header
**Then** the "Start Queue" button is disabled (grayed out)
**And** a tooltip explains: "No pending jobs to execute"

**Given** the queue is processing
**When** I close the application
**Then** jobs continue running on the remote server (tmux persistence)
**And** the queue state (paused/running) is persisted to local database

**Given** I reopen the application after closing it mid-queue
**When** the application starts
**Then** the queue state is restored from database
**And** if the queue was running before close, it resumes automatically (Epic 3 reconciliation)

**And** queue state changes are atomic (no race conditions between pause/resume clicks)
**And** UI button state reflects backend queue state accurately
**And** keyboard shortcut: S (start/pause toggle) for power users

**Technical Notes:**

- New Tauri commands:
  - `start_queue_processing() -> Result<(), String>` (already in Story 2.4)
  - `pause_queue_processing() -> Result<(), String>`
  - `resume_queue_processing() -> Result<(), String>`
  - `get_queue_state() -> Result<QueueState, String>` (returns "idle", "running", "paused")
- Backend: use Arc<Mutex<QueueState>> in AppState to track queue state
- Frontend: enhance QueuePanel.svelte with button controls
- Persist queue state to database: `UPDATE metadata SET value='paused' WHERE key='queue_state'`
- Resume on startup: check metadata table, if queue_state='running' → auto-resume

**FRs Fulfilled:** FR152 (start queue), FR153 (pause queue), FR154 (resume queue), FR160 (resume after reopen)

---

### Story 2.6: Job State Reconciliation - Triple Redundancy Validation

As a system architect,
I want to validate job state using triple redundancy (SQLite → State File → tmux check),
So that job status is accurate even if one source fails or is corrupted.

**Acceptance Criteria:**

**Given** a job completed on the remote server
**When** the backend queries job status
**Then** the reconciliation follows this priority chain:

1. **PRIMARY:** Query server SQLite DB via SSH
2. **FALLBACK:** Parse state file if SQLite unavailable
3. **INFERENCE:** Check tmux session existence if both fail
4. **ERROR:** Mark as "state lost" if all sources fail

**Given** the server SQLite database is healthy
**When** I query job status
**Then** the backend executes:

```bash
ssh user@host "sqlite3 ~/.solverpilot-server/server.db \
  'SELECT status, exit_code, completed_at FROM jobs WHERE id=<job_id>'"
```

**And** the returned status is trusted (primary source)
**And** state file and tmux checks are skipped (optimization)

**Given** the SQLite query fails (database locked, corrupted, or missing)
**When** the FALLBACK activates
**Then** the backend reads the state file via SSH:

```bash
ssh user@host "cat ~/.solverpilot-server/jobs/<job_id>.status"
```

**And** the JSON is parsed: `{"status": "completed", "exit_code": 0, ...}`
**And** the parsed status is used (fallback source)

**Given** both SQLite and state file are unavailable
**When** the INFERENCE activates
**Then** the backend checks tmux session existence:

```bash
ssh user@host "tmux has-session -t solverpilot_<user>_<job_id_short> 2>/dev/null && echo 'exists' || echo 'missing'"
```

**And** if session exists → infer status = "running"
**And** if session missing → infer status = "unknown" (completed or crashed)

**Given** all three sources fail (SQLite error, state file missing, tmux check fails)
**When** the ERROR state is reached
**Then** the job status is marked as "state lost"
**And** error_message is set: "Unable to determine job state - all sources unavailable"
**And** the user is notified via toast: "Job #<id> state unknown - server may be unreachable"

**Given** I have a job marked as "state lost"
**When** I manually trigger reconciliation (button in UI)
**Then** the reconciliation runs again with fresh queries
**And** if server becomes reachable, the state is recovered

**Given** the wrapper was killed with SIGKILL (no state written)
**When** reconciliation runs
**Then** SQLite shows no update (last status = "running")
**And** state file is missing or outdated
**And** tmux session is missing (killed)
**And** status is inferred as "failed" with error_message: "Job terminated unexpectedly (possible SIGKILL)"

**And** reconciliation logic is reused in Epic 3 (startup reconciliation)
**And** all reconciliation queries have 5-second timeout (avoid hanging on unresponsive server)
**And** reconciliation results are logged for debugging: `tracing::info!("Reconciliation for job {}: SQLite={}, StateFile={}, Tmux={}", ...)`

**Technical Notes:**

- Create Rust module: `src-tauri/src/reconciliation.rs`
- New Tauri command: `reconcile_job_state(job_id: String) -> Result<JobStatus, String>`
- Priority chain implementation:

```rust
async fn reconcile_job_state(job_id: &str) -> Result<JobStatus, String> {
    // 1. Try SQLite
    if let Ok(status) = query_server_db(job_id).await {
        return Ok(status);
    }
    // 2. Try state file
    if let Ok(status) = parse_state_file(job_id).await {
        return Ok(status);
    }
    // 3. Try tmux inference
    if let Ok(exists) = check_tmux_session(job_id).await {
        return Ok(if exists { JobStatus::Running } else { JobStatus::Unknown });
    }
    // 4. Error state
    Err("State lost".to_string())
}
```

- Use tokio::time::timeout for 5-second query timeout
- Log all reconciliation attempts with tracing

**FRs Fulfilled:** FR161 (crash recovery), FR162 (detect partially-completed queues), FR163 (recovery status indicators), Architecture requirement (reconciliation priority chain)

---

## Epic 2 Summary

**Stories Created:** 6 stories
**Total FRs Covered:** 9 FRs (FR155-FR157, FR159-FR163) + Architecture requirements

**Story Breakdown:**

- Story 2.1: Bash wrapper with trap EXIT (Architecture)
- Story 2.2: Server SQLite DB schema (Architecture)
- Story 2.3: Wrapper deployment via SSH (Architecture)
- Story 2.4: Sequential queue execution backend (FR155, FR156, FR159)
- Story 2.5: Start/Pause/Resume controls (FR152-FR154, FR160)
- Story 2.6: Triple redundancy reconciliation (FR161-FR163, Architecture)

**Implementation Order:** Sequential (2.1 → 2.2 → 2.3 → 2.4 → 2.5 → 2.6)
**Dependencies:** Each story builds on previous, no forward dependencies

**Research-Driven:** ✅ Hybrid solution from technical research (wrapper + SQLite + state files)
**Reliability Target:** 99.99% state capture (only SIGKILL edge case)

**Ready for Development:** ✅ All stories have clear acceptance criteria, technical notes, and architecture mappings

---

## Epic 3: Startup Reconciliation & "Walk Away" Confidence

**Epic Goal:** Users can close their laptop, go to meetings, or even force-quit the app, then return hours or days later to see exactly what happened while they were away.

**Value Delivered:** Users gain "walk away confidence" - the core emotional promise of Beta 1. They see a Startup Resume Screen showing "3 completed while you were away, 1 failed (here's why), 5 pending." This is the make-or-break trust moment.

**FRs Covered:** FR159-FR163 + UX requirements (Startup Resume Screen, trust-building Tier 1 patterns) (Total: 5 FRs + UX)

---

### Story 3.1: Startup Reconciliation Engine - Multi-Job State Query

As a researcher,
I want the system to automatically reconcile all job states when I reopen the application,
So that I see an accurate picture of what happened while I was away within 10 seconds.

**Acceptance Criteria:**

**Given** I have 10 jobs in the local database with various statuses (3 pending, 2 running, 5 completed)
**When** the application starts and connects to the remote server
**Then** a background reconciliation task launches automatically
**And** the reconciliation queries state for ALL jobs with status IN ('pending', 'running')

**Given** the reconciliation task is running
**When** it processes each job
**Then** it uses the triple redundancy priority chain from Story 2.6:

1. Query server SQLite DB: `SELECT status, exit_code, completed_at FROM jobs WHERE id=?`
2. Fallback to state file: `cat ~/.solverpilot-server/jobs/<job_id>.status`
3. Infer from tmux: `tmux has-session -t solverpilot_<user>_<job_id_short>`
4. Mark as "state lost" if all fail

**Given** a job was marked "running" when I closed the app
**When** reconciliation queries the server
**Then** if the server shows "completed", the local DB is updated:

```sql
UPDATE jobs
SET status='completed',
    completed_at=<remote_completed_at>,
    exit_code=<remote_exit_code>
WHERE id=?
```

**Given** a job was "running" and the server also shows "running"
**When** the tmux session check confirms the session exists
**Then** the job remains "running" (no change needed - still executing)

**Given** a job was "running" but the server shows "failed"
**When** reconciliation detects this
**Then** the local DB is updated to "failed"
**And** the error_message from server DB is copied to local DB

**Given** I have 10 jobs to reconcile
**When** reconciliation processes them
**Then** jobs are queried in parallel (up to 5 concurrent SSH queries for performance)
**And** total reconciliation time is <10 seconds on 50ms latency connection
**And** a progress indicator shows: "Syncing queue state... (7/10 jobs checked)"

**Given** the server is unreachable (network down, SSH timeout)
**When** reconciliation attempts to connect
**Then** after 3 retry attempts with exponential backoff (1s, 3s, 5s)
**And** if all retries fail, reconciliation aborts
**And** the UI shows: "Cannot connect to server - last known state from [timestamp]"
**And** jobs retain their last known status (honest about stale data)

**Given** reconciliation completes successfully
**When** all jobs have been checked
**Then** the reconciliation result is returned with summary:

```rust
struct ReconciliationResult {
    jobs_checked: u32,
    jobs_completed_while_away: u32,
    jobs_failed_while_away: u32,
    jobs_still_running: u32,
    jobs_state_lost: u32,
}
```

**And** all reconciliation queries run in parallel where possible (performance optimization)
**And** reconciliation logic is idempotent (safe to run multiple times)
**And** reconciliation is logged: `tracing::info!("Reconciliation completed: {} checked, {} updated", ...)`

**Technical Notes:**

- New Tauri command: `reconcile_all_jobs() -> Result<ReconciliationResult, String>`
- Called automatically in Tauri setup hook (on app startup)
- Use tokio::spawn for parallel job queries (up to 5 concurrent)
- Use tokio::time::timeout(Duration::from_secs(10), reconcile_task) for overall timeout
- Reuse reconcile_job_state() logic from Story 2.6
- Frontend: show progress toast during reconciliation

**FRs Fulfilled:** FR159 (persist queue across restarts), FR161 (crash recovery), FR162 (detect partially-completed queues)

---

### Story 3.2: Orphaned Session Detection & Conflict Resolution

As a system architect,
I want to detect and resolve orphaned tmux sessions and state conflicts,
So that edge cases like crashed wrappers or server reboots are handled gracefully.

**Acceptance Criteria:**

**Given** a wrapper crashed (SIGKILL) without writing state
**When** reconciliation runs
**Then** the following conflict is detected:

- Local DB: status = "running"
- Server DB: no update (last status = "running" or missing entry)
- State file: missing or outdated
- Tmux session: missing (killed)
  **And** the conflict resolution logic infers: status = "failed", error_message = "Job terminated unexpectedly (wrapper crashed or SIGKILL)"

**Given** a job completed but the wrapper failed to update local DB (app was closed)
**When** reconciliation runs
**Then** the conflict is detected:

- Local DB: status = "running"
- Server DB: status = "completed", exit_code = 0
- State file: status = "completed"
- Tmux session: missing (exited normally)
  **And** the resolution trusts server DB (primary source): update local to "completed"

**Given** a tmux session exists but has no matching DB entry (orphaned session)
**When** reconciliation scans tmux sessions on the server
**Then** it executes: `ssh user@host "tmux list-sessions | grep solverpilot_"`
**And** for each session found, it checks if a matching job_id exists in local DB
**And** if no match found (orphaned), the session is logged as: `tracing::warn!("Orphaned tmux session detected: {}", session_name)`
**And** optionally: show warning in Startup Resume Screen: "1 orphaned session detected on server - may be from previous installation"

**Given** the server was rebooted while jobs were running
**When** reconciliation runs
**Then** the conflict is detected:

- Local DB: status = "running"
- Server DB: status = "running" (stale - process was killed by reboot)
- State file: status = "running" (stale)
- Tmux session: missing (killed by reboot)
  **And** the resolution infers: status = "failed", error_message = "Server reboot detected - job was terminated"

**Given** multiple conflicts exist (3 jobs with different conflict types)
**When** reconciliation processes them
**Then** each conflict is resolved independently using appropriate logic
**And** the Startup Resume Screen shows detailed breakdown: "3 jobs had conflicts - 2 resolved as completed, 1 marked as failed"

**Given** a job is in "state lost" condition (all sources unavailable)
**When** the user manually triggers "Retry Reconciliation" from UI
**Then** reconciliation runs again with fresh queries
**And** if the server becomes reachable, the state is recovered
**And** the UI updates with recovered state

**And** all conflict resolution follows the priority chain (SQLite > State File > tmux)
**And** conflict resolution is conservative (prefer "failed" over "running" when ambiguous)
**And** all resolved conflicts are logged for debugging

**Technical Notes:**

- Enhance reconciliation.rs with conflict detection logic
- Add conflict types enum:

```rust
enum ConflictType {
    WrapperCrashed,      // tmux missing, no state update
    ServerRebooted,      // all sources stale, tmux missing
    LocalStale,          // server ahead of local
    OrphanedSession,     // tmux exists, no DB entry
}
```

- Implement resolution strategy per conflict type
- New command: `list_orphaned_sessions() -> Result<Vec<String>, String>`
- Optional: `cleanup_orphaned_sessions()` command to kill orphaned tmux sessions

**FRs Fulfilled:** FR161 (crash recovery), FR162 (detect conflicts), FR163 (recovery status)

---

### Story 3.3: Startup Resume Screen - Trust Foundation UI

As a researcher,
I want to see a Startup Resume Screen with a clear summary of what happened while I was away,
So that I immediately know if my work progressed successfully and can trust the system.

**Acceptance Criteria:**

**Given** the application was closed with 10 jobs queued (3 pending, 2 running, 5 completed)
**When** I reopen the application after 3 hours
**Then** the Startup Resume Screen appears as a modal overlay before the main UI loads

**Given** the Startup Resume Screen is displayed
**When** I look at the summary section
**Then** I see a large, prominent status summary:

- "3 completed while you were away ✓"
- "1 failed (click to see details)"
- "5 pending"
- "Queue paused - ready to resume"

**Given** the summary shows completed jobs
**When** I view the details
**Then** each completed job displays:

- Benchmark name (e.g., "benchmark_01.py")
- Completion timestamp with relative time: "finished 2h 34m ago"
- Exit code: "exit code 0" (green checkmark)
- Optional: estimated execution duration if available

**Given** the summary shows a failed job
**When** I view the failure details
**Then** I see:

- Benchmark name with red status badge
- Failure timestamp: "failed 1h 12m ago"
- Error message extracted from last 20 lines of log (auto-parsed)
- "View Full Logs" link to open logs panel
- "Retry Now" button for one-click retry

**Given** the summary shows pending jobs
**When** I view the pending section
**Then** I see a count: "5 jobs still pending"
**And** optionally: first 3 benchmark names in pending queue

**Given** the reconciliation detected conflicts or state losses
**When** the Startup Resume Screen displays
**Then** a warning section appears: "⚠️ 1 job state could not be determined - marked as failed"
**And** details explain: "Job 'benchmark_03.py' - wrapper crashed or server rebooted"

**Given** I am viewing the Startup Resume Screen
**When** I want to continue working
**Then** I see two prominent action buttons:

- "Resume Queue" (primary button - green) - starts queue processing
- "Review Queue" (secondary button - gray) - closes modal and shows main queue panel

**Given** I click "Resume Queue"
**When** the button is activated
**Then** the modal closes
**And** the queue processing starts automatically (calls start_queue_processing())
**And** the main UI loads with queue panel showing active execution

**Given** I click "Review Queue"
**When** the button is activated
**Then** the modal closes
**And** the main UI loads with queue panel visible
**And** queue remains paused (user can inspect before starting)

**Given** the server was unreachable during reconciliation
**When** the Startup Resume Screen displays
**Then** an error state is shown:

- "Cannot connect to server"
- "Last known state from [timestamp when app was closed]"
- "Retry Connection" button
- Warning: "Status may be outdated - connect to server for accurate state"

**And** the Startup Resume Screen only appears when there are jobs to reconcile (skip if queue is empty)
**And** the modal has glassmorphism styling (dark overlay, blurred background)
**And** timestamps use relative time ("2h ago") for better readability
**And** the screen is keyboard accessible (Tab navigation, Enter to confirm primary action, Escape to dismiss)

**Technical Notes:**

- Create NEW component: `src/lib/features/queue/StartupResumeScreen.svelte`
- Shown as modal overlay in MainLayout on app startup
- Receives ReconciliationResult from Story 3.1 as prop
- Use Svelte $effect to auto-show modal when reconciliation completes
- Modal styling: z-50, fixed overlay, backdrop-blur-lg, centered card
- Relative timestamps: use `date-fns` or custom Svelte $derived: `$derived(formatRelativeTime(job.completed_at))`
- Primary button triggers IPC: `start_queue_processing()`

**FRs Fulfilled:** FR163 (view recovery status with clear indicators), UX requirement (Startup Resume Screen - Tier 1 trust foundation)

---

### Story 3.4: Queue Locking & User Action Queuing During Sync

As a system architect,
I want to prevent users from modifying the queue during the 5-10 second reconciliation window,
So that race conditions are avoided and state remains consistent.

**Acceptance Criteria:**

**Given** the application starts and reconciliation begins
**When** I try to interact with the queue (click "Queue Selected", "Start Queue", "Remove Job")
**Then** all queue modification actions are disabled (buttons grayed out)
**And** a visual indicator shows: "Syncing queue state..." with a spinner

**Given** reconciliation is in progress (3 out of 10 jobs checked)
**When** the progress indicator updates
**Then** I see: "Syncing queue state... (3/10 jobs checked - 2s remaining)"
**And** the estimated time remaining is calculated based on average query latency

**Given** I try to queue new benchmarks during reconciliation
**When** I press Q or click "Queue Selected"
**Then** the action is queued (not executed immediately)
**And** a toast notification shows: "Action queued - will execute after sync completes"

**Given** reconciliation completes successfully
**When** all jobs have been checked (10/10)
**Then** the queue lock is released
**And** all queued user actions execute in sequence (e.g., queue selected benchmarks)
**And** UI buttons become enabled (no longer grayed out)
**And** the "Syncing..." indicator disappears

**Given** I queued 3 actions during reconciliation (queue benchmarks, reorder job, remove job)
**When** the lock is released
**Then** actions execute in FIFO order:

1. Queue benchmarks
2. Reorder job
3. Remove job
   **And** each action's result is shown via toast notification

**Given** reconciliation fails (server unreachable after retries)
**When** the failure is detected
**Then** the queue lock is released (don't block user indefinitely)
**And** a warning toast shows: "Sync failed - queue state may be outdated. Reconnect to server for accurate status."
**And** queued user actions are discarded (not executed - unsafe with stale state)

**Given** reconciliation takes longer than 10 seconds (slow network or many jobs)
**When** the timeout is reached
**Then** reconciliation is aborted
**And** the queue lock is released
**And** a warning shows: "Sync timed out - some job statuses may be outdated"

**Given** the Startup Resume Screen is visible
**When** reconciliation is in progress
**Then** the modal shows a loading state with progress: "Checking job statuses... (5/10)"
**And** action buttons are disabled until reconciliation completes

**And** queue locking uses a mutex/flag in AppState: `Arc<Mutex<bool>>` for thread-safety
**And** queued actions are stored in a Vec and executed atomically when lock releases
**And** locking mechanism is reused for connection loss scenarios (Epic 6)

**Technical Notes:**

- Add to AppState:

```rust
pub struct AppState {
    // ... existing fields
    pub queue_locked: Arc<Mutex<bool>>,
    pub queued_actions: Arc<Mutex<Vec<QueuedAction>>>,
}
```

- QueuedAction enum:

```rust
enum QueuedAction {
    QueueBenchmarks(Vec<i64>),
    RemoveJob(i64),
    ReorderJob(i64, i32),
    // ... other actions
}
```

- Frontend: disable buttons when queue_locked = true (reactive check)
- Progress indicator component: shows reconciliation progress in real-time
- Timeout mechanism: tokio::time::timeout(Duration::from_secs(10), reconciliation_task)

**FRs Fulfilled:** FR162 (detect conflicts safely), UX requirement (queue operation locking during reconciliation)

---

## Epic 3 Summary

**Stories Created:** 4 stories
**Total FRs Covered:** 5 FRs (FR159-FR163) + UX requirements (Startup Resume Screen, trust patterns)

**Story Breakdown:**

- Story 3.1: Reconciliation engine - multi-job state query (FR159, FR161, FR162)
- Story 3.2: Orphaned session detection & conflict resolution (FR161, FR162, FR163)
- Story 3.3: Startup Resume Screen - trust foundation UI (FR163, UX Tier 1)
- Story 3.4: Queue locking during sync - race condition prevention (FR162, UX)

**Implementation Order:** Sequential (3.1 → 3.2 → 3.3 → 3.4)
**Dependencies:** Each story builds on previous, no forward dependencies

**Emotional Impact - Tier 1 Trust Foundation:**

- ✅ Story 3.3 is the **make-or-break moment** for user trust
- ✅ "That's the moment where I either trust this tool or go back to my janky bash scripts" - Dr. Chen
- ✅ Clear, honest status (never show stale data as current)
- ✅ "3 completed while you were away" = calm accomplishment (not anxiety)

**Edge Cases Covered:**

- Wrapper crashed (SIGKILL) → Detected and marked as "failed"
- Server rebooted → Detected via missing tmux sessions
- Network unreachable → Honest "cannot connect" message with timestamp
- Orphaned tmux sessions → Logged and optionally cleaned up
- Reconciliation timeout → Aborted gracefully, user notified

**Ready for Development:** ✅ All stories have clear acceptance criteria, conflict resolution logic, and UX specifications

---

## Epic 4: Real-Time Monitoring & Progress Visibility

**Epic Goal:** Users can instantly see queue status, monitor job progress in real-time, and review logs without clicking through multiple panels.

**Value Delivered:** Users have calm productivity - they can glance at the screen and immediately understand "3 running • 12 pending • 8 completed" without anxiety about whether work is progressing. No hunting for status.

**FRs Covered:** FR122-FR137 (queue monitoring subset) + UX requirements (always-visible status, glassmorphism, keyboard shortcuts) (Total: 16 FRs + UX)

---

### Story 4.1: Backend Polling - 2-Second Queue State Updates

As a researcher,
I want the backend to automatically poll queue state every 2 seconds and update the UI reactively,
So that I see current job statuses without manually refreshing.

**Acceptance Criteria:**

**Given** the application is open and connected to the remote server
**When** the queue monitoring starts
**Then** a background polling task launches automatically
**And** the task queries queue state every 2 seconds (configurable interval)

**Given** the polling task is running
**When** each 2-second interval elapses
**Then** the backend executes `get_queue_summary()` command which:

1. Queries local database for all jobs with queue_position IS NOT NULL
2. For jobs with status = 'running', queries server DB for updates:
   - `ssh user@host "sqlite3 ~/.solverpilot-server/server.db 'SELECT status, exit_code FROM jobs WHERE id IN (...)'"` (batch query for efficiency)
3. Returns summary with counts:

```rust
struct QueueSummary {
    pending_count: u32,
    running_count: u32,
    completed_count: u32,
    failed_count: u32,
    active_jobs: Vec<Job>, // currently running jobs with details
}
```

**Given** a job status changes on the server (running → completed)
**When** the next polling interval occurs
**Then** the backend detects the change
**And** the local database is updated immediately
**And** the frontend receives the updated QueueSummary via event emission

**Given** the polling task detects a status change
**When** the update occurs
**Then** a Tauri event is emitted: `emit("queue-state-changed", queue_summary)`
**And** the frontend listens to this event and updates reactive state

**Given** I am on a slow network (200ms latency)
**When** polling executes
**Then** batch queries are used (single SSH command for multiple jobs)
**And** polling completes in <1 second even with 10 running jobs

**Given** the server is temporarily unreachable
**When** a polling attempt fails
**Then** the polling task continues retrying every 2 seconds
**And** after 3 consecutive failures, a toast notification shows: "Lost connection to server - retrying..."
**And** the UI continues showing last known state with timestamp: "Last updated: 30s ago"

**Given** the application is minimized or in background
**When** polling continues
**Then** the polling interval remains 2 seconds (no throttling - maintain real-time updates)

**Given** the queue is empty (no jobs)
**When** polling executes
**Then** the backend skips server queries (optimization)
**And** returns empty QueueSummary immediately

**And** polling uses bb8 connection pool for SSH efficiency (reuse connections)
**And** only jobs with status = 'running' are queried on the server (not completed/failed - optimization)
**And** polling task is cancelled gracefully on app shutdown (no orphaned tasks)

**Technical Notes:**

- Create background task in `queue_service.rs`:

```rust
pub async fn start_queue_polling(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        if let Ok(summary) = get_queue_summary().await {
            app.emit_all("queue-state-changed", summary).ok();
        }
    }
}
```

- New Tauri command: `get_queue_summary() -> Result<QueueSummary, String>`
- Start polling in Tauri setup hook (after initial reconciliation)
- Frontend: listen to event in QueuePanel.svelte:

```typescript
import { listen } from '@tauri-apps/api/event';
listen<QueueSummary>('queue-state-changed', event => {
  queueSummary = event.payload;
});
```

**FRs Fulfilled:** FR122 (view live output - foundation), FR134 (see currently running job)

---

### Story 4.2: Always-Visible Queue Status Summary in Panel Header

As a researcher,
I want to see a prominent queue status summary in the panel header at all times,
So that I can glance at the screen and immediately know how many jobs are pending/running/completed without scrolling or clicking.

**Acceptance Criteria:**

**Given** I have 15 jobs in the queue (3 running, 7 pending, 4 completed, 1 failed)
**When** I look at the Queue panel header
**Then** I see a prominent status summary with dot separators:

- "3 running • 7 pending • 4 completed • 1 failed"

**Given** the queue status summary is displayed
**When** the status updates (e.g., a job completes)
**Then** the summary updates reactively within 2 seconds
**And** the transition is smooth (no jarring flash)

**Given** the status includes running jobs
**When** I view the summary
**Then** running jobs are shown first with a green dot: "● 3 running"
**And** the color matches the running status badge color (green-500)

**Given** the status includes failed jobs
**When** I view the summary
**Then** failed jobs are highlighted in red: "● 1 failed"
**And** clicking on "1 failed" filters the queue to show only failed jobs (Story 1.5 integration)

**Given** the queue is empty (no jobs)
**When** I view the Queue panel header
**Then** the summary shows: "Queue empty"
**And** the "Start Queue" button is disabled

**Given** I have only pending jobs (no running)
**When** the queue is paused
**Then** the summary shows: "10 pending (paused)"
**And** a small pause icon appears next to the count

**Given** I have 50+ completed jobs and 5 pending
**When** the summary displays
**Then** only relevant counts are shown: "5 pending • 50+ completed"
**And** the "50+" indicates truncation (optimization for readability)

**Given** the summary shows counts
**When** I hover over a count (e.g., "3 running")
**Then** a tooltip appears with benchmark names:

- "benchmark_01.py"
- "benchmark_05.py"
- "benchmark_12.py"

**And** the summary uses monospace tabular numerals for digit alignment
**And** the summary is always visible (sticky header or fixed position)
**And** the summary styling uses glassmorphism (bg-slate-900/75, 2px backdrop-blur)
**And** the summary updates are throttled to prevent excessive re-renders (max 1 update per 500ms)

**Technical Notes:**

- Enhance QueuePanel.svelte header section
- Use Svelte $derived for reactive summary:

```typescript
let summary = $derived({
  running: jobs.filter(j => j.status === 'running').length,
  pending: jobs.filter(j => j.status === 'pending').length,
  completed: jobs.filter(j => j.status === 'completed').length,
  failed: jobs.filter(j => j.status === 'failed').length,
});
```

- Tooltip component: show first 5 job names on hover (use existing Tooltip component or create new)
- Clickable counts: onClick filters queue (integrate with Story 1.5 filtering)
- Styling: font-variant-numeric: tabular-nums for monospace digits

**FRs Fulfilled:** FR134 (see currently running job), FR151 (view all jobs with statuses - summary view)

---

### Story 4.3: Real-Time Progress Indicators - Elapsed Time & [x/y] Parsing

As a researcher,
I want to see elapsed time counters and progress parsing ([x/y]) for running jobs,
So that I can estimate how far along my jobs are without guessing.

**Acceptance Criteria:**

**Given** a job is running for 3 hours and 24 minutes
**When** I view the job in the Queue panel
**Then** I see elapsed time displayed as: "Running for 3h 24m"
**And** the time updates every second (client-side counter - no backend cost)

**Given** a job just started (running for 5 seconds)
**When** I view the elapsed time
**Then** it displays: "Running for 5s"

**Given** a job has been running for over 24 hours
**When** I view the elapsed time
**Then** it displays: "Running for 1d 3h 15m"

**Given** a job outputs progress indicators like `[45/100]` in its logs
**When** the backend parses the latest log lines
**Then** the progress is extracted and stored in database:

- `progress_current = 45`
- `progress_total = 100`

**Given** the progress data is available (45/100)
**When** I view the running job
**Then** I see a progress indicator: "[45/100]" next to the elapsed time
**And** a progress bar is displayed showing 45% completion (visual indicator)

**Given** the solver does not output progress markers
**When** I view the running job
**Then** I see only elapsed time: "Running for 2h 15m"
**And** NO fake progress bar is shown (honest progress only - no fake ETAs)

**Given** the backend detects a new progress marker in logs
**When** the parsing occurs
**Then** the regex matches patterns: `\[(\d+)/(\d+)\]` or `(\d+)/(\d+)`
**And** the progress updates immediately in the UI (via polling event)

**Given** multiple running jobs are displayed
**When** I view the Queue panel
**Then** each running job shows its own elapsed time counter
**And** all counters tick independently (client-side JavaScript setInterval per job)

**Given** a job completes
**When** the final elapsed time is calculated
**Then** the total duration is shown: "Completed in 4h 32m"
**And** the elapsed time counter stops (no longer updating)

**And** elapsed time is calculated client-side (no backend load - uses job.started_at timestamp)
**And** progress parsing only updates when new log lines contain [x/y] patterns (not every log line)
**And** progress bars use accessible colors (green for 0-50%, yellow for 50-75%, blue for 75-100%)
**And** NO ETAs are shown (optimization problems unpredictable - honest progress only per UX requirement)

**Technical Notes:**

- Client-side elapsed time calculation:

```typescript
let elapsedTime = $derived(() => {
  if (job.status === 'running' && job.started_at) {
    const now = Date.now();
    const start = new Date(job.started_at).getTime();
    return formatDuration(now - start); // "3h 24m"
  }
  return null;
});
```

- Use JavaScript setInterval for live counter updates (1-second interval)
- Backend: add progress parsing in log polling (Epic 4 Story 4.4):

```rust
fn parse_progress(log_line: &str) -> Option<(i32, i32)> {
    let re = Regex::new(r"\[(\d+)/(\d+)\]").ok()?;
    let caps = re.captures(log_line)?;
    Some((caps[1].parse().ok()?, caps[2].parse().ok()?))
}
```

- Progress bar component: `<ProgressBar current={45} total={100} />`
- Update database when progress changes: `UPDATE jobs SET progress_current=?, progress_total=? WHERE id=?`

**FRs Fulfilled:** FR124 (parse progress indicators), FR125 (display elapsed time), FR126 (detect completion)

---

### Story 4.4: Live Log Streaming for Selected Job

As a researcher,
I want to see live log output from the currently selected running job,
So that I can debug issues or monitor solver output in real-time.

**Acceptance Criteria:**

**Given** I have a job running on the remote server
**When** I click on the job in the Queue panel
**Then** the job is selected (highlighted with border)
**And** the right panel (Logs panel) immediately shows the latest logs from that job

**Given** the selected job is running
**When** the log streaming starts
**Then** the backend polls logs every 2 seconds via SSH:

- `ssh user@host "tail -n 50 ~/.solverpilot-server/logs/<job_id>.log"`

**Given** new log lines are written by the solver
**When** the next polling interval occurs (2 seconds)
**Then** the new lines are fetched
**And** the UI appends new lines to the log panel (incremental update, not full replace)
**And** the log panel auto-scrolls to the bottom (latest logs visible)

**Given** the log output is 500 lines long
**When** I view the logs
**Then** only the last 50 lines are fetched initially (tail -n 50)
**And** I can scroll up to load earlier logs (infinite scroll or "Load More" button)

**Given** I click "Load More" to see earlier logs
**When** the request is made
**Then** the backend fetches the previous 50 lines:

- `ssh user@host "tail -n 100 ~/.solverpilot-server/logs/<job_id>.log | head -n 50"`
  **And** the earlier lines are prepended to the log panel (scroll position preserved)

**Given** the solver outputs logs rapidly (100 lines/second)
**When** the 2-second polling occurs
**Then** up to 200 new lines are fetched (throttled to prevent UI thrash)
**And** a warning appears: "High log volume - updates throttled to 2-second batches"

**Given** I select a different job
**When** the selection changes
**Then** the log panel immediately switches to the new job's logs
**And** the previous job's log streaming stops (cleanup - prevent memory leak)

**Given** I select a completed job
**When** I view the logs
**Then** the full log history is available (not just last 50 lines)
**And** no live polling occurs (job is finished - static logs)

**Given** the log file does not exist on the server
**When** I try to view logs
**Then** the log panel shows: "No logs available for this job"

**Given** the log streaming encounters an SSH error
**When** the error occurs
**Then** the log panel shows the last successfully fetched logs
**And** a warning appears: "Lost connection - logs may be outdated"

**And** log streaming uses ANSI color stripping (raw logs may have ANSI codes from solvers)
**And** logs are displayed in monospace font (Consolas, Monaco, 'Courier New')
**And** log panel has dark background (bg-slate-950) for better readability
**And** long lines wrap (no horizontal scrolling required)
**And** logs are searchable (Ctrl+F highlights matches - browser native search)

**Technical Notes:**

- New Tauri command: `stream_job_logs(job_id: String) -> Result<Vec<String>, String>` (returns last N lines)
- Frontend: create LogsPanel.svelte component in right panel
- Use Svelte $effect to start/stop streaming when selected job changes:

```typescript
$effect(() => {
  if (selectedJobId) {
    const interval = setInterval(async () => {
      const logs = await api.streamJobLogs(selectedJobId);
      appendLogsToPanel(logs);
    }, 2000);
    return () => clearInterval(interval); // cleanup
  }
});
```

- ANSI stripping: use `strip-ansi` library or regex: `log_line.replace(/\x1b\[[0-9;]*m/g, '')`
- Auto-scroll: check if user manually scrolled up, if not → auto-scroll to bottom

**FRs Fulfilled:** FR122 (view live log output), FR123 (stream logs from tmux), FR131 (view complete log history), FR132 (export logs - add button), FR133 (preserve logs in DB)

---

### Story 4.5: Job Timeout Detection & Long-Running Warnings

As a researcher,
I want to be warned when jobs run abnormally long or exceed configured timeouts,
So that I can detect stuck jobs or infinite loops before they waste server resources.

**Acceptance Criteria:**

**Given** I configure a global job timeout of 6 hours in settings
**When** a job runs for longer than 6 hours
**Then** the backend detects the timeout
**And** the job is automatically killed (tmux session terminated)
**And** the job status is set to "killed" with error_message: "Job exceeded timeout of 6 hours"

**Given** a job has been running for 8 hours (abnormally long)
**When** the polling detects this
**Then** a warning badge appears on the job: "⚠ Running for 8h (unusually long)"
**And** a toast notification shows: "Job 'benchmark_01.py' has been running for 8h - check if it's stuck"

**Given** I have not configured a timeout
**When** jobs run
**Then** no automatic killing occurs (timeout disabled by default)
**And** warnings still appear after 6 hours (soft warning, no action)

**Given** I want to set a per-job timeout (not global)
**When** I queue a benchmark
**Then** I can optionally specify: "Max execution time: 2 hours"
**And** this overrides the global timeout for that specific job

**Given** a job is killed due to timeout
**When** I view the job details
**Then** the status shows "Killed" with red badge
**And** the error message explains: "Job exceeded configured timeout of 6h and was automatically terminated"
**And** I can retry the job with a higher timeout or no timeout

**Given** a job runs for 30 minutes (normal range for my benchmarks)
**When** I view the job
**Then** NO warning is shown (30 minutes is not abnormally long)

**Given** the average job duration in my history is 45 minutes
**When** a new job runs for 4 hours
**Then** the system detects this is >5x the average
**And** a warning appears: "⚠ This job is taking longer than usual (avg: 45m)"

**Given** I want to ignore timeout warnings for a specific job
**When** I configure the job as "long-running expected"
**Then** no warnings appear even if it runs for 24+ hours

**And** timeout detection runs during the 2-second polling cycle (no separate task)
**And** timeout configuration is stored in settings (global) and per-job metadata
**And** killing a job uses: `ssh user@host "tmux kill-session -t <session_name>"`
**And** job history tracking enables "average duration" calculation for warnings

**Technical Notes:**

- Add to settings/config: `job_timeout_hours: Option<u32>` (None = disabled)
- Add to jobs table: `timeout_override_hours: Option<u32>`
- Timeout detection in polling loop:

```rust
if let Some(timeout) = get_effective_timeout(&job) {
    let elapsed = now - job.started_at;
    if elapsed > timeout {
        kill_job(job.id).await?;
        update_status(job.id, "killed", "Exceeded timeout").await?;
    }
}
```

- New Tauri command: `kill_job(job_id: String) -> Result<(), String>`
- Warning threshold: hardcoded at 6 hours OR 5x average duration (whichever is longer)
- Average duration calculation: `SELECT AVG(julianday(completed_at) - julianday(started_at)) * 24 FROM jobs WHERE status='completed'`

**FRs Fulfilled:** FR135 (detect job timeout), FR136 (warn about long-running jobs), FR137 (notifications for state changes)

---

## Epic 4 Summary

**Stories Created:** 5 stories
**Total FRs Covered:** 16 FRs (FR122-FR137) + UX requirements

**Story Breakdown:**

- Story 4.1: Backend polling every 2 seconds (FR122, FR134 - infrastructure)
- Story 4.2: Always-visible status summary (FR134, FR151 - glanceable UI)
- Story 4.3: Progress indicators - elapsed time + [x/y] (FR124, FR125, FR126)
- Story 4.4: Live log streaming (FR122, FR123, FR131-FR133)
- Story 4.5: Timeout detection & warnings (FR135-FR137)

**Implementation Order:** Sequential (4.1 → 4.2 → 4.3 → 4.4 → 4.5)
**Dependencies:** Each story builds on previous, no forward dependencies

**Calm Productivity - UX Philosophy:**

- ✅ Glanceable status: "3 running • 12 pending" always visible (no hunting)
- ✅ Honest progress: Elapsed time + [x/y] parsing, NO fake ETAs
- ✅ Real-time updates: 2-second polling, client-side counters
- ✅ Visual hierarchy: Running (prominent) > Pending (subdued) > Completed (collapsed)
- ✅ Glassmorphism: 2px backdrop-blur for 60fps resize performance

**Performance Optimizations:**

- Batch queries for multiple running jobs (single SSH command)
- Client-side elapsed time counters (no backend cost)
- Throttled log updates (max 200 lines per 2-second batch)
- Skip server queries for completed/failed jobs (only poll "running")
- bb8 connection pool reuse (avoid SSH handshake overhead)

**Ready for Development:** ✅ All stories have clear acceptance criteria, polling logic, and UX specifications

---

## Epic 5: Failed Job Handling & Queue Resilience

**Epic Goal:** Users can handle job failures gracefully - failed jobs don't break the queue, error messages are clear, and retry is one-click away.

**Value Delivered:** Users feel accomplishment instead of frustration. They see "Job 3 failed (here's why), but Jobs 4-10 kept running" and can retry with confidence. Failed jobs prove system resilience rather than creating anxiety.

**FRs Covered:** FR184-FR191 + UX requirements (failed job indicators, one-click retry, raw error logs) (Total: 8 FRs + UX)

---

### Story 5.1: Queue State Machine - Failed Jobs Don't Block Execution

As a researcher,
I want failed jobs to automatically transition to "failed" status and allow the queue to continue with the next pending job,
So that one failing job doesn't block my entire queue from completing.

**Acceptance Criteria:**

**Given** I have 10 jobs queued (positions 1-10, all pending)
**When** the queue starts executing job #3
**Then** job #3 status changes to "running"

**Given** job #3 fails with exit code 1
**When** the wrapper captures the failure and updates server DB
**Then** the server DB records:

- `status = 'failed'`
- `exit_code = 1`
- `completed_at = <timestamp>`

**Given** the backend polling detects job #3 is failed
**When** the local DB is updated
**Then** job #3 transitions: running → failed
**And** the queue state machine immediately selects job #4 (next pending)
**And** job #4 starts execution automatically (no pause, no user intervention)

**Given** jobs #1-2 completed successfully, job #3 failed, job #4 is now running
**When** I view the queue
**Then** I see:

- Jobs #1-2: status "completed" (green badges)
- Job #3: status "failed" (red badge)
- Job #4: status "running" (green, currently executing)
- Jobs #5-10: status "pending" (blue badges, waiting)

**Given** multiple jobs fail in sequence (jobs #3, #5, #7 all fail)
**When** each failure is detected
**Then** the queue continues executing:

- Job #3 fails → Job #4 starts
- Job #4 completes → Job #5 starts
- Job #5 fails → Job #6 starts
- Job #6 completes → Job #7 starts
- Job #7 fails → Job #8 starts
  **And** the queue never pauses or stops due to failures

**Given** all remaining jobs in the queue fail
**When** the last job (job #10) fails
**Then** the queue execution stops (no more pending jobs)
**And** a toast notification shows: "Queue completed - 3 succeeded, 7 failed"

**Given** a job fails due to network error during rsync
**When** the failure is detected
**Then** the job status is set to "failed"
**And** error_message is populated: "Failed to sync project files: <rsync_error_details>"
**And** the next job starts (queue continues despite rsync failure)

**Given** max_concurrent = 1 (Beta 1 sequential execution)
**When** a job fails
**Then** the single execution slot is immediately freed
**And** the next pending job starts without delay

**And** the state machine transitions are atomic (no race conditions)
**And** failed jobs remain in the queue (visible for review - don't disappear)
**And** the queue can be filtered to show "only failed" jobs (Story 1.5 integration)

**Technical Notes:**

- Enhance queue_service.rs execution loop:

```rust
loop {
    if let Some(job) = get_next_pending_job().await? {
        execute_job(job).await?;
        // Poll for completion
        while job_is_running(job.id).await? {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        // Job completed or failed - state machine continues to next
    } else {
        break; // No more pending jobs
    }
}
```

- State machine logic: On job completion/failure detection → immediately call `get_next_pending_job()`
- Failed jobs keep queue_position (don't renumber) - allows retry to restore original position

**FRs Fulfilled:** FR184 (prevent cascade failures - failed job doesn't stop queue)

---

### Story 5.2: Failure Indicators & Error Message Extraction

As a researcher,
I want to see clear failure indicators and understand why jobs failed,
So that I can diagnose issues and fix them before retrying.

**Acceptance Criteria:**

**Given** a job fails with exit code 1
**When** I view the job in the Queue panel
**Then** the job displays a red status badge: "Failed"
**And** the badge uses color (red-500) + icon (X) + text ("Failed") for WCAG AAA triple encoding

**Given** a failed job has error output in its logs
**When** the backend detects the failure
**Then** the error message is extracted from the last 20 lines of the log:

- Search for common error patterns: "Error:", "Exception:", "FAILED:", "Traceback"
- Extract the first matching line as error_message
- Store in database: `UPDATE jobs SET error_message=? WHERE id=?`

**Given** the error message was successfully extracted
**When** I hover over the failed job badge
**Then** a tooltip appears showing the error message snippet (first 100 characters)

**Given** I click on a failed job
**When** the job details expand
**Then** I see:

- Full error message (not truncated)
- Exit code: "exit code 1" (or the actual code)
- "View Full Logs" button to open raw logs in right panel
- "Retry" button for one-click retry (Story 5.3)

**Given** a job fails due to solver crash (no error message in logs)
**When** the error extraction runs
**Then** a generic error message is used: "Job failed with exit code 1 (no error message found in logs)"

**Given** I have 5 failed jobs in the queue
**When** I filter the queue to "Show Only Failed" (Story 1.5)
**Then** only the 5 failed jobs are visible
**And** each shows its error message in the list view (not just on hover)

**Given** a job fails due to timeout (Story 4.5)
**When** the failure is recorded
**Then** the error_message is: "Job exceeded configured timeout of 6h and was terminated"
**And** the status badge shows "Killed" (orange badge, distinct from "Failed" red)

**Given** the error message is very long (500+ characters)
**When** I view it in the job details
**Then** the message is displayed with scrolling (not truncated)
**And** I can copy the error message to clipboard (copy button)

**Given** I export the queue to CSV (future feature)
**When** the export includes failed jobs
**Then** the error_message column contains the full extracted error

**And** failed jobs persist in the queue indefinitely (don't auto-clear)
**And** failed jobs can be manually cleared via "Clear Completed Jobs" which removes both completed AND failed (Story 1.4 enhancement)
**And** error extraction regex is configurable for custom solver error patterns

**Technical Notes:**

- Error extraction logic in job.rs:

```rust
fn extract_error_message(log_lines: &[String]) -> Option<String> {
    let error_patterns = vec![
        r"(?i)error:\s*(.+)",
        r"(?i)exception:\s*(.+)",
        r"(?i)failed:\s*(.+)",
        r"Traceback \(most recent call last\):",
    ];
    // Search last 20 lines for first match
    for line in log_lines.iter().rev().take(20) {
        for pattern in &error_patterns {
            if let Some(caps) = Regex::new(pattern).ok()?.captures(line) {
                return Some(caps.get(1).map_or(line.clone(), |m| m.as_str().to_string()));
            }
        }
    }
    None
}
```

- Call error extraction when job status transitions to "failed"
- Frontend: enhance JobListItem component with expandable error details
- Clipboard copy: use `navigator.clipboard.writeText()` or Tauri clipboard API

**FRs Fulfilled:** FR190 (view failure reasons for all failed jobs)

---

### Story 5.3: One-Click Retry & Batch Retry Operations

As a researcher,
I want to retry failed jobs with a single click or keyboard shortcut,
So that I can quickly re-run jobs that failed due to transient issues without manually re-queuing.

**Acceptance Criteria:**

**Given** I have a failed job in the queue
**When** I click on the job to select it
**Then** the job is highlighted (selected state)

**Given** a failed job is selected
**When** I press the R key (retry keyboard shortcut)
**Then** the retry action triggers immediately

**Given** the retry action is triggered
**When** the backend processes the retry
**Then** a new job entry is created:

- New job_id (UUID or timestamp-based)
- Same benchmark_path as the failed job
- Same project_id, command arguments, environment variables
- Status: "pending"
- Queue_position: appended to end of queue (highest position + 1)
  **And** the original failed job remains in the queue (unchanged)

**Given** the retry creates a new job
**When** I view the queue
**Then** I see:

- Original failed job: status "failed", queue_position 3
- New retry job: status "pending", queue_position 11 (at end)
  **And** the retry job has a badge: "Retry of #3" (links to original)

**Given** I want to retry a failed job immediately (skip queue)
**When** I click "Retry" button
**Then** I see two options:

- "Add to Queue" (default - appends to end)
- "Retry Now" (advanced - pauses queue, runs immediately, then resumes)

**Given** I select "Retry Now"
**When** the retry executes
**Then** the queue is paused
**And** the retry job starts immediately (bypasses queue_position)
**And** after completion/failure, the queue resumes from where it paused

**Given** I have 5 failed jobs in the queue
**When** I filter to "Show Only Failed" and select all 5
**Then** a "Retry All" button appears in the panel header

**Given** I click "Retry All" for 5 failed jobs
**When** the batch retry executes
**Then** 5 new pending jobs are created
**And** all 5 are appended to the queue (positions 11-15)
**And** a toast notification shows: "5 jobs added to queue for retry"

**Given** I retry a job that previously failed due to timeout
**When** I configure the retry
**Then** I can optionally increase the timeout: "Retry with 12h timeout (was 6h)"

**Given** a retry job completes successfully
**When** I view the queue
**Then** the original failed job still shows "failed" (historical record)
**And** the retry job shows "completed"
**And** both jobs are linked (clicking one highlights the other)

**And** retry preserves all original job configuration (args, env vars, timeout)
**And** keyboard shortcut R works on multi-select (retry all selected failed jobs)
**And** retry confirmation modal for "Retry Now" to prevent accidental queue disruption

**Technical Notes:**

- New Tauri command: `retry_job(job_id: i64, retry_mode: RetryMode) -> Result<Job, String>`
- RetryMode enum:

```rust
enum RetryMode {
    AddToQueue,     // Append to end (default)
    RetryNow,       // Pause queue, run immediately
}
```

- New command: `retry_all_failed() -> Result<Vec<Job>, String>` (batch retry)
- Link retry jobs to original:
  - Add column: `retried_from_job_id INTEGER NULL` (foreign key to original)
  - Add column: `retry_count INTEGER DEFAULT 0` (track retry attempts)
- Frontend: "Retry All" button in QueuePanel header when failed jobs exist
- Keyboard shortcut handler: detect R key press on selected jobs

**FRs Fulfilled:** FR185 (configure retry behavior - manual mode), FR186 (retry capability - manual)

---

### Story 5.4: Auto-Retry with Exponential Backoff & Quarantine

As a researcher,
I want the option to automatically retry failed jobs with exponential backoff,
So that transient failures (network glitches, temporary server issues) are retried without manual intervention.

**Acceptance Criteria:**

**Given** I enable auto-retry in settings
**When** I configure the setting
**Then** I can set:

- Enable/disable auto-retry: toggle (default: disabled)
- Max retry attempts: 1-5 (default: 3)
- Backoff delays: "immediate, 10s, 30s" (default exponential pattern)

**Given** auto-retry is enabled with max_attempts = 3
**When** a job fails for the first time
**Then** the system automatically creates a retry job
**And** the retry is scheduled immediately (0 delay)
**And** retry_count = 1 is recorded

**Given** the first retry also fails
**When** the failure is detected
**Then** a second retry is scheduled with 10-second delay
**And** retry_count = 2 is recorded
**And** a toast notification shows: "Job 'benchmark_01.py' failed (attempt 2/3) - retrying in 10s..."

**Given** the second retry also fails
**When** the failure is detected
**Then** a third retry is scheduled with 30-second delay
**And** retry_count = 3 is recorded

**Given** the third retry also fails (retry_count = 3, max_attempts = 3)
**When** the failure is detected
**Then** the job is quarantined:

- Status: "quarantined"
- No more automatic retries
- error_message: "Job failed 3 times - quarantined to prevent infinite retries"
  **And** a toast notification shows: "Job 'benchmark_01.py' failed 3 times - quarantined"

**Given** a job is quarantined
**When** I view the job in the queue
**Then** the status badge shows "Quarantined" (yellow/orange badge)
**And** I can manually retry if I want (quarantine doesn't prevent manual retry)

**Given** auto-retry is enabled
**When** a job fails due to timeout
**Then** auto-retry is NOT triggered (timeout failures are not transient - user intervention needed)

**Given** auto-retry is enabled
**When** a job fails with specific exit codes (e.g., 127 = command not found)
**Then** auto-retry is NOT triggered (permanent errors - won't be fixed by retry)
**And** the job is immediately marked as "failed - permanent error"

**Given** I want to customize which failures trigger auto-retry
**When** I configure settings
**Then** I can specify:

- "Retry only network/connection errors" (exit codes: connection timeout, rsync failures)
- "Retry all failures" (default)
- "Never auto-retry" (manual only)

**Given** a retry job is scheduled with 10-second delay
**When** the delay elapses
**Then** the retry job is added to the queue as pending
**And** normal queue execution picks it up when its turn comes

**Given** I have 10 failed jobs that all triggered auto-retry
**When** the retries are queued
**Then** the queue doesn't become overwhelmed (retries respect queue_position and sequential execution)

**And** auto-retry delays are configurable per settings
**And** quarantine threshold is configurable (default 3 attempts)
**And** quarantined jobs can be "unquarantined" manually (reset retry_count, allow retry)
**And** all retry attempts are logged for audit trail

**Technical Notes:**

- Add to settings/config:

```rust
struct AutoRetryConfig {
    enabled: bool,
    max_attempts: u32,
    backoff_delays_seconds: Vec<u32>, // [0, 10, 30]
    retry_on_exit_codes: Option<Vec<i32>>, // None = all, Some([...]) = specific
}
```

- Enhance queue_service.rs to check auto-retry config on failure
- Schedule delayed retry using tokio::time::sleep:

```rust
if should_auto_retry(&job, &config) {
    let delay = config.backoff_delays_seconds[job.retry_count as usize];
    tokio::time::sleep(Duration::from_secs(delay)).await;
    retry_job(job.id, RetryMode::AddToQueue).await?;
}
```

- Add "quarantined" status to jobs table status CHECK constraint
- New command: `unquarantine_job(job_id: i64) -> Result<(), String>` (reset retry_count)
- Frontend: settings panel for auto-retry configuration

**FRs Fulfilled:** FR185 (configure retry behavior), FR186 (auto-retry with backoff), FR191 (quarantine repeatedly-failing jobs)

---

## Epic 5 Summary

**Stories Created:** 4 stories
**Total FRs Covered:** 8 FRs (FR184-FR191) + UX requirements

**Story Breakdown:**

- Story 5.1: Queue state machine - failed jobs don't block (FR184)
- Story 5.2: Failure indicators & error extraction (FR190)
- Story 5.3: One-click retry & batch retry (FR185, FR186 manual)
- Story 5.4: Auto-retry with backoff & quarantine (FR185, FR186, FR191)

**Implementation Order:** Sequential (5.1 → 5.2 → 5.3 → 5.4)
**Dependencies:** Each story builds on previous, no forward dependencies

**Resilience Philosophy - "Accomplishment, Not Anxiety":**

- ✅ Failed jobs prove resilience (queue continues executing)
- ✅ Clear failure reasons (extracted from last 20 log lines)
- ✅ One-click retry (R keyboard shortcut, "Retry" button)
- ✅ Optional auto-retry with intelligent backoff (0s, 10s, 30s)
- ✅ Quarantine after 3 failures (prevent infinite loops)

**UX Patterns:**

- Failed jobs persist (don't disappear - visible for review)
- Red badge + error message (triple encoding: color + icon + text)
- "View Full Logs" link (access raw solver output)
- "Retry All Failed" for batch recovery
- Toast notifications for retry attempts: "Job failed (attempt 2/3) - retrying in 10s..."

**Edge Cases Covered:**

- Timeout failures → No auto-retry (user intervention needed)
- Permanent errors (exit code 127) → No auto-retry (won't be fixed)
- Quarantined jobs → Manual retry still available
- Multiple failures in sequence → Queue continues (3, 5, 7 all fail → 4, 6, 8 still run)

**Ready for Development:** ✅ All stories have clear acceptance criteria, state machine logic, and retry strategies

---

## Epic 6: Connection Resilience & Auto-Recovery

**Epic Goal:** Users can trust that network disconnects won't break their work - the system automatically reconnects and transparently resumes operation.

**Value Delivered:** Users have "walk away confidence" even with unstable university VPNs or laptop sleep. They see "SSH connection lost 5 min ago, attempting reconnect..." and watch it transparently recover. Jobs continue on server regardless.

**FRs Covered:** Architecture requirements (bb8 connection pooling, health checks, auto-reconnect) + UX requirements (dual-channel connection indicators, reconnection messaging) (Total: Architecture + UX)

---

### Story 6.1: bb8 Connection Pooling - 10x SSH Performance Improvement

As a system architect,
I want SSH connections managed through bb8 connection pool,
So that SSH operations reuse existing connections and achieve ~10x performance improvement over repeated handshakes.

**Acceptance Criteria:**

**Given** the application starts and needs to establish SSH connections
**When** the bb8 pool is initialized
**Then** a connection pool is created with configuration:

- `max_connections: 10` (up to 10 concurrent SSH sessions)
- `idle_timeout: 5 minutes` (keep connections alive for 5 minutes of inactivity)
- `connection_timeout: 10 seconds` (fail if can't establish within 10s)

**Given** the pool is initialized
**When** the first SSH operation is requested (e.g., rsync, execute command)
**Then** a new SSH connection is established via russh
**And** the connection is added to the pool
**And** the operation completes using this connection

**Given** a connection exists in the pool
**When** a second SSH operation is requested (within 5-minute idle timeout)
**Then** the existing connection is reused from the pool
**And** NO new SSH handshake occurs (performance optimization)
**And** the operation completes in <50ms (vs ~500ms for new connection)

**Given** I execute 100 SSH commands in sequence
**When** all commands use the connection pool
**Then** only 1 SSH handshake occurs (first connection)
**And** subsequent 99 commands reuse the pooled connection
**And** total time is ~5 seconds (100 × 50ms) vs ~50 seconds without pooling (100 × 500ms)

**Given** a pooled connection has been idle for 5 minutes
**When** the idle timeout is reached
**Then** the connection is closed gracefully
**And** the pool size decreases by 1
**And** the next SSH operation creates a new connection

**Given** a pooled connection fails (network error, server unreachable)
**When** the failure is detected
**Then** the connection is removed from the pool
**And** the pool automatically attempts to create a new connection
**And** if the new connection fails, the error is propagated to the caller

**Given** 10 connections are active (pool at max capacity)
**When** an 11th SSH operation is requested
**Then** the operation waits for a connection to become available
**And** once a connection is freed, the operation proceeds
**And** a timeout of 30 seconds applies (returns error if can't acquire connection)

**Given** the application shuts down
**When** cleanup occurs
**Then** all pooled connections are closed gracefully
**And** no orphaned SSH connections remain on the server

**And** pool statistics are logged: "Connection pool: 3 active, 7 idle, 0 waiting"
**And** pool health is monitored (Story 6.2 uses pool for health checks)
**And** connection errors include pool state for debugging: "Failed to acquire connection (pool: 10/10 active, 5 waiting)"

**Technical Notes:**

- Enhance `src-tauri/src/ssh/pool.rs` (already exists from Alpha - extend for bb8)
- Implement bb8::ManageConnection trait for SshManager:

```rust
#[async_trait]
impl bb8::ManageConnection for SshConnectionManager {
    type Connection = SshSession;
    type Error = String;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        // Establish russh connection with aws-lc-rs crypto
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        // Validate connection with lightweight echo command
        conn.execute("echo 'ping'").await
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        // Check if connection is broken
        conn.is_closed()
    }
}
```

- Create pool in AppState:

```rust
pub struct AppState {
    // ... existing fields
    pub ssh_pool: Arc<bb8::Pool<SshConnectionManager>>,
}
```

- All SSH operations (rsync, command execution, queries) use pool.get().await

**FRs Fulfilled:** Architecture requirement (bb8 connection pooling, ~10x performance improvement)

---

### Story 6.2: Health Checks & Connection Status Tracking

As a researcher,
I want the system to continuously monitor SSH connection health,
So that connection failures are detected within 10 seconds and I'm immediately aware of connectivity issues.

**Acceptance Criteria:**

**Given** the application is connected to the remote server
**When** health check monitoring starts
**Then** a background task executes health checks every 10 seconds
**And** the health check uses a lightweight echo command: `echo 'health_check_ping'`

**Given** a health check executes successfully
**When** the echo command returns within 2 seconds
**Then** the connection status is set to "connected"
**And** the connection status state is updated in AppState:

```rust
pub connection_status: Arc<Mutex<ConnectionStatus>> // "connected" | "reconnecting" | "disconnected"
```

**Given** a health check fails (timeout or SSH error)
**When** the failure is detected
**Then** the connection status is set to "reconnecting"
**And** auto-reconnect logic is triggered (Story 6.3)

**Given** 3 consecutive health checks fail
**When** the third failure is detected
**Then** the connection status is set to "disconnected"
**And** a Tauri event is emitted: `emit("connection-status-changed", "disconnected")`

**Given** the connection status changes (connected → reconnecting → disconnected)
**When** each transition occurs
**Then** a Tauri event is emitted with the new status
**And** the frontend updates UI indicators reactively (Story 6.4)

**Given** health checks are running
**When** queue polling also needs SSH access (Story 4.1)
**Then** health checks and queue polling are independent (separate connections from pool)
**And** health check failures don't block queue polling (avoid mixing concerns)

**Given** the connection is "reconnecting"
**When** queue polling attempts to execute
**Then** queue polling is paused (don't query stale data)
**And** client-side elapsed time counters continue (no backend needed - Story 4.3)

**Given** the connection recovers (health check succeeds after failures)
**When** the status changes to "connected"
**Then** a "connection-recovered" event is emitted
**And** queue polling resumes automatically
**And** reconciliation is triggered (Epic 3 - sync state after disconnect)

**And** health check interval is configurable (default 10 seconds)
**And** health checks use bb8 pool (reuse connections for efficiency)
**And** health check errors are logged: `tracing::warn!("Health check failed: {}", error)`
**And** connection status is persisted (shown in UI even after app restart until next check)

**Technical Notes:**

- Create background task in `ssh/pool.rs`:

```rust
pub async fn start_health_checks(app: AppHandle, pool: Arc<bb8::Pool<SshConnectionManager>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        match execute_health_check(&pool).await {
            Ok(_) => update_connection_status(&app, "connected"),
            Err(_) => trigger_reconnect(&app).await,
        }
    }
}

async fn execute_health_check(pool: &bb8::Pool<SshConnectionManager>) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    tokio::time::timeout(
        Duration::from_secs(2),
        conn.execute("echo 'health_check_ping'")
    ).await.map_err(|_| "Health check timeout".to_string())?
}
```

- New Tauri command: `get_connection_status() -> Result<String, String>` (returns "connected" | "reconnecting" | "disconnected")
- Event emission: `app.emit_all("connection-status-changed", status)`
- Frontend listener: `listen('connection-status-changed', updateConnectionIndicator)`

**FRs Fulfilled:** Architecture requirement (10-second health checks, connection status tracking)

---

### Story 6.3: Auto-Reconnect with Exponential Backoff

As a researcher,
I want the system to automatically reconnect after SSH connection loss with exponential backoff,
So that transient network issues are recovered without manual intervention.

**Acceptance Criteria:**

**Given** a health check fails and connection status is "reconnecting"
**When** auto-reconnect logic is triggered
**Then** the system attempts to reconnect with 3 retries:

- Retry 1: immediate (0 seconds delay)
- Retry 2: 10 seconds delay
- Retry 3: 30 seconds delay

**Given** the first reconnect attempt executes
**When** a new SSH connection is established
**Then** the bb8 pool creates a fresh connection
**And** a health check is immediately executed to validate the connection
**And** if successful, connection status changes to "connected"

**Given** the first reconnect attempt fails
**When** the failure is detected
**Then** a toast notification shows: "Reconnecting to server... (attempt 1/3)"
**And** the system waits 10 seconds before retry 2

**Given** the second reconnect attempt also fails
**When** the failure is detected
**Then** a toast notification shows: "Reconnecting to server... (attempt 2/3)"
**And** the system waits 30 seconds before retry 3

**Given** all 3 reconnect attempts fail
**When** the final failure is detected
**Then** connection status is set to "disconnected"
**And** a toast notification shows: "Connection lost - unable to reconnect. Click 'Retry Connection' to try again."
**And** auto-reconnect stops (manual intervention required via UI button)

**Given** reconnection succeeds on retry 2
**When** the connection is re-established
**Then** connection status changes to "connected"
**And** a toast notification shows: "Reconnected to server ✓"
**And** reconciliation is triggered automatically (Epic 3 - sync job states)
**And** queue polling resumes (if queue was active)

**Given** the connection is lost while a job is running
**When** reconnection occurs
**Then** the job continues running on the server (tmux persistence)
**And** after reconciliation, the job status is updated in local DB
**And** a toast shows: "Reconnected - 1 job completed while disconnected" (reconciliation result)

**Given** the user manually closes the laptop (sleep mode)
**When** the laptop wakes up and network is restored
**Then** health checks detect the connection is stale
**And** auto-reconnect triggers automatically
**And** the user sees seamless recovery: "Reconnected ✓"

**Given** the server reboots while the client is connected
**When** the connection is lost
**Then** auto-reconnect waits for the server to come back online
**And** retries continue until server is reachable or max attempts reached

**Given** the network is permanently down (router unplugged)
**When** all retry attempts fail
**Then** the UI clearly shows "Disconnected" status
**And** queue operations are blocked (grayed out buttons)
**And** last known state is shown with timestamp: "Last updated: 2 minutes ago"

**And** reconnection attempts are logged: `tracing::info!("Reconnect attempt {}/3 failed: {}", attempt, error)`
**And** successful reconnection triggers event: `emit("connection-recovered", reconciliation_result)`
**And** reconnection logic reuses Epic 3 reconciliation for state sync

**Technical Notes:**

- Implement in `ssh/pool.rs`:

```rust
async fn trigger_reconnect(app: &AppHandle) {
    let backoff_delays = vec![0, 10, 30]; // seconds
    for (attempt, delay) in backoff_delays.iter().enumerate() {
        tokio::time::sleep(Duration::from_secs(*delay)).await;

        app.emit_all("reconnect-attempt", attempt + 1).ok();

        match pool.get().await {
            Ok(conn) => {
                if execute_health_check(&pool).await.is_ok() {
                    update_connection_status(app, "connected");
                    trigger_reconciliation(app).await;
                    return;
                }
            },
            Err(_) => continue,
        }
    }
    // All attempts failed
    update_connection_status(app, "disconnected");
}
```

- Integrate with Epic 3 reconciliation: call `reconcile_all_jobs()` after successful reconnect
- Frontend: listen to "reconnect-attempt" events to show toast progress

**FRs Fulfilled:** FR187 (auto-reconnect SSH after transient failures), FR189 (detect server reboots and re-establish), Architecture requirement (3 retries with exponential backoff)

---

### Story 6.4: Dual-Channel Connection Indicators & Transparent Recovery UI

As a researcher,
I want to see connection status through both ambient awareness (header glow) and active visibility (text indicator),
So that I can monitor connectivity at a glance and be informed of recovery progress.

**Acceptance Criteria:**

**Given** the connection is healthy (status = "connected")
**When** I view the application
**Then** I see dual-channel indicators:

- **Ambient awareness:** Header bottom border with green glow (border-b-2 border-green-500/40)
- **Active visibility:** Text indicator in Queue panel header: "● Connected" (green dot + text)

**Given** the connection is lost and reconnecting (status = "reconnecting")
**When** the UI updates
**Then** I see:

- **Ambient awareness:** Header bottom border with yellow glow (border-yellow-500/40)
- **Active visibility:** "⚠ Reconnecting..." (yellow warning icon + text with spinner animation)

**Given** reconnection is in progress
**When** retry attempts occur
**Then** the text indicator updates: "⚠ Reconnecting (attempt 2/3)..."
**And** a subtle spinner animation plays next to the text

**Given** all reconnection attempts fail (status = "disconnected")
**When** the UI updates
**Then** I see:

- **Ambient awareness:** Header bottom border with red glow (border-red-500/40)
- **Active visibility:** "✗ Disconnected" (red X icon + text)
  **And** a "Retry Connection" button appears in the Queue panel header

**Given** I click the "Retry Connection" button
**When** the button is activated
**Then** manual reconnection is triggered (bypasses auto-reconnect limit)
**And** the same retry logic executes (immediate, 10s, 30s delays)

**Given** the connection is reconnecting
**When** I hover over the status indicator
**Then** a tooltip appears: "Lost connection 45s ago - attempting to reconnect..."
**And** the tooltip shows time since disconnect

**Given** reconnection succeeds
**When** the connection is re-established
**Then** a toast notification appears: "Reconnected to server ✓"
**And** if jobs completed while disconnected: "Reconnected - 2 jobs completed while disconnected"
**And** the border glow transitions smoothly from yellow → green (CSS transition)

**Given** the connection has been disconnected for 5 minutes
**When** I view the Queue panel
**Then** the queue UI shows: "Last updated: 5 minutes ago"
**And** job statuses display last known state (honest about stale data)
**And** elapsed time counters continue ticking (client-side - proof jobs are still running)

**Given** I lose connection while reviewing logs
**When** the disconnect occurs
**Then** logs panel shows: "Connection lost - showing last known logs"
**And** log streaming stops (no stale updates)
**And** I can still scroll through cached logs (client-side data)

**Given** the connection recovers after 2 minutes
**When** reconciliation completes
**Then** the toast shows reconciliation results:

- "Reconnected - all jobs still running ✓"
- "Reconnected - 1 job completed, 3 still running"
- "Reconnected - 2 jobs failed while disconnected (view details)"

**And** border glow uses 40% opacity for subtle ambient awareness (not distracting)
**And** peripheral vision catches color change (green → yellow → red)
**And** active users see explicit text status (no need to look away from queue)
**And** connection indicators are always visible (header is sticky/fixed)
**And** color transitions are smooth (CSS transition: border-color 300ms ease)

**Technical Notes:**

- Enhance Header.svelte component:

```svelte
<script>
  let connectionStatus = $state<'connected' | 'reconnecting' | 'disconnected'>('connected');

  $effect(() => {
    listen('connection-status-changed', event => {
      connectionStatus = event.payload;
    });
  });

  let borderColor = $derived(() => {
    switch (connectionStatus) {
      case 'connected':
        return 'border-green-500/40';
      case 'reconnecting':
        return 'border-yellow-500/40';
      case 'disconnected':
        return 'border-red-500/40';
    }
  });
</script>

<header class="border-b-2 {borderColor} transition-colors duration-300">
  <!-- header content -->
</header>
```

- Enhance QueuePanel header with connection text indicator
- Toast notifications via existing toast store
- "Retry Connection" button calls new command: `force_reconnect() -> Result<(), String>`
- Spinner animation: use existing spinner component or CSS animation

**FRs Fulfilled:** FR129 (reconnect to running job after interruption), UX requirements (dual-channel connection indicators, reconnection toast notifications, honest stale data messaging)

---

## Epic 6 Summary

**Stories Created:** 4 stories
**Total Coverage:** Architecture requirements + UX requirements (bb8 pooling, health checks, auto-reconnect, dual-channel UI)

**Story Breakdown:**

- Story 6.1: bb8 connection pooling - 10x performance (Architecture)
- Story 6.2: Health checks every 10 seconds (Architecture)
- Story 6.3: Auto-reconnect with exponential backoff (FR187, FR189, Architecture)
- Story 6.4: Dual-channel connection indicators & recovery UI (FR129, UX)

**Implementation Order:** Sequential (6.1 → 6.2 → 6.3 → 6.4)
**Dependencies:** Each story builds on previous, no forward dependencies

**Walk Away Confidence - Even with Unstable Networks:**

- ✅ bb8 pooling: 10 concurrent connections, 5-minute idle timeout
- ✅ Health checks: 10-second interval, lightweight echo command
- ✅ Auto-reconnect: 3 retries (0s, 10s, 30s backoff)
- ✅ Dual-channel UI: Border glow (ambient) + text indicator (active)

**UX Philosophy - "Transparency Over False Reassurance":**

- Honest problems better than fake calm
- Peripheral vision catches border glow (green/yellow/red)
- Active users see explicit status ("● Connected", "⚠ Reconnecting...", "✗ Disconnected")
- Jobs continue on server regardless (tmux persistence)
- Reconciliation shows what happened: "2 jobs completed while disconnected"

**Performance Benefits:**

- ~10x improvement: 50ms vs 500ms for SSH operations (connection reuse)
- 100 commands: 5 seconds vs 50 seconds (single handshake vs 100 handshakes)
- Pool management: auto-cleanup idle connections (5-minute timeout)

**Edge Cases Covered:**

- Laptop sleep → Auto-reconnect on wake
- Server reboot → Retry until server is back online
- Permanent network loss → Manual "Retry Connection" button
- Jobs running during disconnect → Continue on server, sync on reconnect
- Multiple simultaneous operations → Pool manages concurrency (max 10 connections)

**Ready for Development:** ✅ All stories have clear acceptance criteria, pool configuration, and transparency patterns
