---
stepsCompleted: [1, 2, 3, 4, 6, 7, 8, 9, 10, 11]
inputDocuments:
  - 'docs/index.md'
  - 'docs/project-structure.md'
  - 'docs/architecture-patterns.md'
  - 'docs/technology-stack.md'
  - 'docs/data-models-backend.md'
  - 'docs/ipc-commands-integration.md'
  - 'docs/ui-component-inventory-frontend.md'
  - 'docs/integration-architecture.md'
  - 'docs/state-management-patterns-frontend.md'
  - 'docs/development-guide.md'
  - 'docs/deployment-guide.md'
  - 'docs/source-tree-analysis.md'
  - 'docs/existing-documentation-inventory.md'
  - 'docs/user-provided-context.md'
documentCounts:
  briefCount: 0
  researchCount: 0
  brainstormingCount: 0
  projectDocsCount: 14
workflowType: 'prd'
lastStep: 11
---

# Product Requirements Document - SolverPilot

**Author:** Yanis
**Date:** 2026-01-07

## Executive Summary

### Vision & Purpose

**SolverPilot** is a desktop application that eliminates manual SSH workflows for running computationally demanding Python scripts on remote servers. Built for researchers and engineers executing resource-intensive workloads (optimization, ML training, simulations, data processing), SolverPilot automates the entire workflow from "select files" to "get results."

**The Problem:**

Running Python scripts on remote servers manually wastes 5-10 minutes per job:

- Repeated SSH logins and authentication
- Manual file transfers (rsync/git)
- tmux session setup (or risk losing work on disconnect)
- Dependency installation and troubleshooting
- No tracking system for what's run or what's pending

**The Solution:**

A cross-platform desktop app (Tauri 2) where users:

1. Configure server connection once (setup wizard)
2. Select Python files to run
3. Click "run" - SolverPilot handles everything

**Alpha v1.0 Status:**

Core automation is working:

- ✅ SSH connection pooling (persistent, efficient connections)
- ✅ Automatic file sync (rsync) and dependency detection (tree-sitter AST parsing)
- ✅ Environment management (uv package manager - faster, modern)
- ✅ tmux session management (jobs survive disconnects)
- ✅ Real-time job monitoring with progress tracking
- ✅ Local database (SQLite) for project/job history

**Planned for Beta:**

- Job queue system (multiple jobs, sequential execution)
- Automatic result file download (CSV, outputs from server)

### What Makes This Special

**Core Value Proposition:** "Queue remote Python jobs, walk away, get results - no SSH required."

**Key Differentiators:**

1. **Universal Automation** - Works with any Python script (optimization, ML, simulations), not framework-specific
2. **Time Savings** - Eliminates 5-10 min/job of manual SSH overhead
3. **Intelligent Dependency Management** - tree-sitter parses actual Python AST (handles complex imports regex can't)
4. **Modern Tooling** - uv for fast dependency resolution, russh for pure-Rust SSH
5. **Resilient Execution** - tmux sessions survive network disconnects
6. **Self-Hosted Alternative** - To Gurobi Cloud (for optimization) or cloud notebooks (Colab/SageMaker)

**Target User Persona:**

**The Computational Researcher/Engineer:**

- Runs resource-demanding Python workloads multiple times per week
- Has access to remote compute server (better specs than laptop)
- Values automation over SSH expertise
- Currently suffers through manual SSH workflows
- **Jobs-to-be-Done:** Run computationally expensive scripts on remote server without SSH hassle

**Competitive Landscape:**

- **Primary alternative:** Manual SSH + tmux + rsync (universal, painful)
- **Niche alternatives:** Gurobi Cloud (optimization only), cloud notebooks (Colab, SageMaker - different workflow)
- **SolverPilot advantage:** Self-hosted, framework-agnostic, desktop-native

## Project Classification

**Technical Type:** Desktop App (Tauri 2 - cross-platform)
**Primary Domain:** Developer Tool (workflow automation)
**Secondary Domain:** Scientific Computing (optimization, simulations, ML)
**Complexity Level:** Medium
**Project Context:** Brownfield - Alpha v1.0 MVP in development

**Technology Stack:**

- **Platform:** Windows, macOS, Linux
- **Frontend:** Svelte 5 + TypeScript + Vite + TailwindCSS 4
- **Backend:** Rust (Edition 2021) + Tauri 2
- **Key Libraries:**
  - russh (pure Rust SSH client with connection pooling via bb8)
  - SQLx (compile-time checked SQLite queries)
  - tree-sitter-python (accurate AST-based dependency parsing)
  - uv (modern, fast Python package manager)

**Architecture Principles:**

- **Boring Technology** - Proven tools (SQLite, rsync, tmux) over experimental
- **Connection Pooling** - Reuse SSH connections (bb8) vs naive open/close per job
- **Local-First** - SQLite on client, no server-side database required
- **Framework Agnostic** - Works with any Python, not coupled to specific libraries

**Target Users:**

- **Primary:** Self (optimization researcher) - dogfooding for real workflow
- **Secondary:** Researchers/engineers running computational Python workloads remotely
- **Expansion:** Anyone needing SSH automation for resource-intensive scripts

## Success Criteria

### User Success

**Core User Success Metric:** Users can launch Python files and get results without ever manually SSH-ing to the server.

**Key Success Indicators:**

1. **Zero Manual SSH** - Users never need to open terminal and `ssh user@server` for launching Python workloads
2. **Set and Forget** - Queue files, close laptop, return to completed jobs with downloaded results
3. **No Lost Work** - tmux persistence means SSH disconnects don't kill running jobs
4. **Easy Progress Tracking** - Real-time logs and progress monitoring make it easy to follow execution
5. **Dependency Confidence** - Python scripts run without "ModuleNotFoundError" surprises (for standard static imports)

**User Success Moment:**

When a user realizes they haven't manually SSH'd in a week because SolverPilot handled everything.

**Success Measurement:**

- **Observable metric:** All optimization jobs tracked in SolverPilot database = 100% coverage
- **Behavior validation:** Zero manual SSH sessions to server for Python job execution
- **Proxy metric:** User's terminal history shows no `ssh user@server` followed by Python execution

### Business Success

**Primary Success (Dogfooding):**

- **Metric:** Developer (Yanis) uses SolverPilot for 100% of remote Python job execution
- **Validation:** "Never SSH manually to server for launching Python files"
- **Timeline:** Achieved when Beta 2 is complete (queue + result download working)

**Secondary Success (Adoption):**

- **Near-term:** Share with 3-5 colleagues/researchers running computational workloads
- **Adoption metric:** Users choose SolverPilot over manual SSH for majority of jobs
- **Value proposition validated:** "Saves 5-10 min/job" translates to ~1 hour/week for active users

**Long-term Vision:**

- Researchers and engineers in optimization, ML, simulation domains discover SolverPilot as the go-to alternative to manual SSH workflows
- Self-hosted alternative to Gurobi Cloud gains traction in communities wanting control

### Technical Success

**SolverPilot Reliability Target: 100%**

- **Definition:** SolverPilot code functions correctly 100% of the time (SSH pooling, file sync, job submission, monitoring)
- **Excludes:** Python script failures (bugs in user code, server-side issues like disk full, OOM)
- **Measurement:** Jobs successfully submitted, monitored, and tracked - even if Python script itself fails
- **Quality bar:** No SolverPilot crashes, SSH connection failures, dependency detection errors, or file sync failures

**Job Success Rate: Measured but Not Guaranteed**

- **Definition:** Percentage of Python scripts that complete successfully
- **Factors outside control:** Server resource issues, bugs in user Python code, network between servers
- **Tracking:** Log and report job failures separately from SolverPilot failures

**Dependency Detection Accuracy: 100% for Static Imports**

- **Target:** tree-sitter AST parsing correctly identifies all standard Python imports
- **Scope:** Static imports using standard Python patterns:
  - `import module`
  - `from module import name`
  - `from module import name1, name2`
  - Multiline imports with parentheses
  - Conditional imports in standard `if` blocks

- **Known Limitations (Out of Scope for Beta):**
  - Dynamic imports: `__import__(f'module_{x}')`
  - Imports inside `try/except` blocks (edge cases)
  - Imports in `exec()` or `eval()` statements
  - Plugins loaded at runtime via non-standard mechanisms

- **Validation:** Scripts run without missing dependencies when packages are available via uv
- **Failure mode:** Only acceptable failure is when package doesn't exist in PyPI (user error) or uses dynamic import patterns

**Performance: Sufficient (Not Optimized Yet)**

- **Current state:** SSH connection pooling provides fast-enough execution
- **Philosophy:** Reliability over speed for Beta releases
- **Future optimization:** Performance benchmarking and optimization deferred to post-Beta (v1.0+)

**Technical Success Moment:**

When the system handles 100 consecutive jobs without a single SolverPilot failure (SSH, dependency, sync, monitoring).

### Measurable Outcomes

**Alpha → Beta Progression:**

- ✅ **Alpha (Current):** Core automation working (SSH pooling, dependency detection, tmux, monitoring)
- 🎯 **Beta 1:** Queue system implemented (multiple jobs, sequential execution)
- 🎯 **Beta 2:** Automatic result file download (CSV, outputs retrieved from server)

**Beta 1 Exit Criteria:**

"A user can select 10 Python benchmark files, queue them all sequentially, monitor progress in real-time, and have all jobs complete successfully - without manual SSH intervention."

**Validation Metrics (Beta 1):**

- 50+ queued jobs executed successfully
- Zero manual SSH required for entire workflow
- SolverPilot reliability: 100% (no SSH, sync, or monitoring failures)
- Job success rate: Measured and reported (target: >90% for well-tested scripts)

**Beta 2 Exit Criteria:**

"After queuing and running jobs, results (CSV files, outputs) are automatically downloaded to local machine without user intervention."

**Validation Metrics (Beta 2):**

- 10+ jobs with result downloads tested successfully
- Result files retrieved correctly (matching expected outputs)
- Download success rate: >95% (accounting for network issues)
- File detection accuracy validated (no missing outputs, no false positives)

**Implementation Strategy (Beta 2 Result Download):**

_Two potential approaches under consideration:_

1. **tree-sitter output detection:** Analyze Python script for file write operations, predict output files
2. **Folder monitoring:** User specifies output directory pattern (e.g., `results/*.csv`), download all files in that location after job completes

_Decision deferred to Beta 2 planning phase._

## Product Scope

### MVP - Phased Beta Release

**Beta 1 Scope (Queue System):**

**Must Have:**

1. **Job Queue Management**
   - Select multiple Python files for execution
   - Queue jobs in database (FIFO ordering)
   - Single worker process (sequential execution, one job at a time)
   - Queue status tracking (pending, running, completed, failed)
   - Start/pause/resume queue controls
   - Job cancellation (kill running job via tmux)

2. **Queue UI**
   - Visual queue display (list of pending/running/completed jobs)
   - Queue position indicators
   - Estimated time remaining (based on average job duration)
   - Manual job reordering (priority adjustment)

3. **Core Automation (Already Working)**
   - SSH connection pooling ✅
   - Automatic file sync (rsync) ✅
   - Dependency detection (tree-sitter) + uv management ✅
   - tmux session persistence ✅
   - Real-time job monitoring with progress ✅
   - Local database tracking ✅

**Beta 1 Exit Criteria:**

- User successfully queues and runs 50+ jobs without manual SSH
- Queue system handles job sequencing correctly
- SolverPilot reliability: 100% (zero failures in SSH, sync, queue management)

---

**Beta 2 Scope (Result Download):**

**Must Have:**

1. **Automatic Result Download**
   - Detect or configure output files created by Python scripts
   - Download results to local machine after job completion
   - Organize downloads by project/job/benchmark
   - Handle various file sizes (1KB to 1GB+)
   - Handle network interruptions gracefully (resume partial downloads)

2. **Result Download Configuration**
   - User specifies output file patterns (e.g., `results/*.csv`, `output/*.json`)
   - OR tree-sitter detection of file write operations (exploratory)
   - File size limits (warn on >100MB downloads)
   - Storage location configuration

3. **Download Monitoring**
   - Progress indicators for large file downloads
   - Success/failure notifications
   - Retry mechanism for failed downloads

**Beta 2 Exit Criteria:**

- 10+ jobs with successful result downloads
- Files retrieved match expected outputs
- Download success rate >95% (accounting for network issues)
- User workflow: Queue → Run → Auto-download → Review results locally

---

### Growth Features (Post-Beta / v1.0+)

**Failure Recovery & Resilience:**

- Automatic retry on SSH connection failures
- File sync retry with exponential backoff
- Result download resume on network interruption
- Queue recovery after SolverPilot crash

**Performance Optimization Track:**

- Benchmark SSH connection overhead vs manual
- Optimize dependency detection speed
- Profile and optimize file sync performance
- Connection pool tuning

**Enhanced Monitoring:**

- Job history visualization
- Execution time trends
- Success/failure analytics
- Server resource monitoring (CPU, memory, disk usage)

**Testing & Quality:**

- Automated test suite for core workflows
- Synthetic test jobs for edge case validation
- CI/CD integration for regression testing

**Multi-User Support:**

- Shared server configurations
- Team job history
- Collaborative project management

### Vision (v2.0 - Future)

**Advanced Visualization:**

- Graph optimization results (gap tracking over iterations)
- Render result images directly in app
- Interactive result exploration
- Comparative analysis across job runs

**Enhanced Automation:**

- Parallel job execution (multiple tmux sessions)
- Conditional job chains (run B after A completes)
- Scheduled job execution
- Smart retry on transient failures

**Ecosystem Integration:**

- Jupyter notebook integration
- CI/CD pipeline integration
- Cloud provider support (AWS, Azure, GCP)
- Docker container execution support

## User Journeys

### Journey 1: Current Reality - Manual SSH Hell

**User:** Yanis (Computational Researcher)  
**Context:** Monday morning, 10 new Gurobi optimization benchmarks to run  
**Current Workflow (Manual SSH):**

Yanis opens his terminal with a sigh. He has 10 new optimization benchmarks to run on the server today, and he knows it's going to be painful.

**Benchmark 1:**

```bash
# 1. SSH into server (wait for auth)
ssh yanis@compute-server
# Password prompt... enters password... connected

# 2. Create directory for today's run
mkdir -p /home/yanis/optimization/2026-01-07

# 3. Exit SSH, prepare files locally
exit

# 4. Use rsync to send files
rsync -avz benchmark_01.py pyproject.toml yanis@compute-server:/home/yanis/optimization/2026-01-07/
# Wait... wait... uploaded

# 5. SSH back in
ssh yanis@compute-server
# Password again...

# 6. Navigate to directory
cd /home/yanis/optimization/2026-01-07

# 7. Install dependencies
uv sync
# Wait for package resolution...

# 8. Start tmux (HOPEFULLY remembering this time!)
tmux new -s bench01

# 9. Run the script
python benchmark_01.py
# It's running!

# 10. Detach from tmux
# Ctrl+B, D

# 11. Exit SSH
exit
```

**Time elapsed:** 10 minutes (if everything goes smoothly)

**Benchmark 2:**
Yanis repeats the process. SSH, rsync, SSH again, tmux, run... 10 more minutes.

**Benchmark 5:**
By the fifth benchmark, Yanis realizes something horrible: he forgot to start tmux for Benchmark 3. His SSH connection had dropped, and the job was killed. He has to start over.

**Benchmark 8:**
Yanis's laptop battery dies. When he reconnects, he can't remember which benchmarks have finished and which are still running. He SSHs in, checks `tmux ls`, tries to remember which session is which...

**End of day:**

- 15 hours spent on what should have been 5 minutes of actual compute configuration
- 3 jobs lost due to forgotten tmux sessions
- No centralized tracking of what ran, what failed, what's pending
- Results scattered across the server, manually downloaded one by one with `scp`
- Yanis's terminal history is a graveyard of `ssh`, `rsync`, `tmux attach`, repeated endlessly

**Pain Points:**

- **Repetitive SSH authentication** (10+ times/day)
- **Forgetting tmux** = lost work on disconnect
- **No queue system** = manual sequencing
- **No progress tracking** = constant SSH-ing back to check status
- **Manual file management** = rsync every time, results scattered
- **Cognitive overhead** = remembering what's run, what's pending, which tmux session is which

### Journey 2: First Liberation - Discovering SolverPilot Alpha

**User:** Yanis  
**Context:** First time using SolverPilot  
**Date:** Week after building Alpha v1.0

Yanis opens SolverPilot for the first time. The UI greets him with a clean, modern interface.

**Step 1: Setup Wizard**
A wizard dialog appears: "Welcome to SolverPilot! Let's configure your remote server."

Fields:

- **Server hostname:** `compute-server`
- **Username:** `yanis`
- **SSH key:** File picker → selects `~/.ssh/id_ed25519`
- **Remote base directory:** `/home/yanis/optimization`
- **Python version:** `3.12`

Click "Test Connection" → Green checkmark: "✓ Connected successfully"  
Click "Save Configuration"

**Time elapsed:** 2 minutes (one-time setup)

**Step 2: Create Project**
Main interface shows empty state: "No projects yet. Create one to get started."

Click "+ New Project"

- **Project name:** `gurobi-benchmarks`
- **Python version:** `3.12` (pre-filled from config)
- **Local directory:** File picker → selects `/Users/yanis/projects/gurobi-benchmarks`

Click "Create" → Project appears in left sidebar

**Step 3: Add Benchmark**
Click project → Right panel shows "Benchmarks (0)"

Click "+ Add Benchmark"

- File picker opens → Navigate to `benchmark_01.py`
- Select file → SolverPilot analyzes dependencies in real-time
  - "Analyzing imports... Found: `gurobipy`, `numpy`, `pandas`"
  - "Checking existing environment... Installing packages via uv..."
  - Progress bar: Installing packages... ✓ Complete

Benchmark appears in list with:

- Name: `benchmark_01.py`
- Status: Ready to run
- Dependencies: 3 packages installed

**Step 4: Run First Job**
Click "Run" button on `benchmark_01.py`

Background (user doesn't see, but it's happening):

1. SolverPilot rsyncs project files to remote server
2. Opens SSH connection from pool (persistent connection)
3. Creates tmux session `solver-pilot-job-1`
4. Starts `python benchmark_01.py` in tmux
5. Begins tailing logs

**What Yanis sees:**

- Job panel switches to "Running" view
- Real-time log output streams:
  ```
  Initializing Gurobi model...
  Adding variables... [1/100]
  Adding variables... [50/100]
  Adding constraints... [1/500]
  ```
- Elapsed time counter: 00:02:15... 00:02:16... 00:02:17...
- Progress indicator: "Adding constraints [312/500]"

**Step 5: Close Laptop**
Yanis's laptop battery is low. He thinks "Oh no, I'll lose the job!"

Then remembers: SolverPilot runs jobs in tmux on the server. It's safe.

He closes his laptop and goes to lunch.

**Step 6: Return and Check Status**
Opens laptop 30 minutes later. Opens SolverPilot.

Job status automatically updates:

- Status: ✓ Completed
- Elapsed time: 00:18:32
- Final log lines:
  ```
  Optimal solution found
  Objective value: 42.7
  Writing results to results.csv
  ```

**Aha Moment:**
"I didn't SSH once. I didn't forget tmux. I didn't manually rsync. It just... worked."

**Time saved:** 10 minutes of manual SSH workflow → 30 seconds of clicking "Run"

### Journey 3: Beta 1 Vision - The Queue Changes Everything

**User:** Yanis  
**Context:** Monday morning, 10 new benchmarks to run (same as Journey 1)  
**Date:** After Beta 1 release (queue system implemented)

Yanis opens SolverPilot with 10 new benchmark files ready to run.

**Step 1: Select Multiple Benchmarks**
In the Benchmarks panel, he:

1. Selects all 10 files at once (shift-click)
2. Right-click → "Add to Queue"

All 10 benchmarks appear in the Queue panel:

```
Queue (10 pending)
1. benchmark_01.py - Pending
2. benchmark_02.py - Pending
3. benchmark_03.py - Pending
...
10. benchmark_10.py - Pending
```

**Step 2: Start Queue**
Clicks big green "Start Queue" button.

Queue begins processing:

```
Queue (1 running, 9 pending)
1. benchmark_01.py - Running (00:02:15 elapsed) [Progress: 45/100]
2. benchmark_02.py - Pending
3. benchmark_03.py - Pending
...
```

Real-time updates:

- Current job shows live logs and progress
- Pending jobs show estimated start time based on average job duration
- Completed jobs show checkmarks and final status

**Step 3: Walk Away**
Yanis checks the queue: "Average job time: ~15 minutes. Total queue time: ~2.5 hours."

He thinks: "In the old workflow, I'd have to manually start each one. Now I can just... leave."

He closes his laptop and goes to work on other tasks.

**Step 4: Return After Lunch**
Opens SolverPilot 3 hours later.

Queue status:

```
Queue (10 completed)
✓ benchmark_01.py - Completed (00:14:32) - Success
✓ benchmark_02.py - Completed (00:16:05) - Success
✓ benchmark_03.py - Completed (00:15:18) - Success
...
✓ benchmark_09.py - Completed (00:17:42) - Success
✗ benchmark_10.py - Completed (00:00:45) - Failed (ModuleNotFoundError)
```

**Step 5: Review Results**
Clicks on failed job (`benchmark_10.py`) to see logs:

```
ModuleNotFoundError: No module named 'cvxpy'
```

"Ah, I forgot to add cvxpy to pyproject.toml."

Fixes dependency, re-queues `benchmark_10.py`, clicks "Start Queue" again.

**Aha Moment:**
"I queued 10 jobs, went to lunch, came back to 9 completed jobs. In the old workflow, this would have taken me all day of manual SSH sessions. Now it's 30 seconds of setup and 0 minutes of babysitting."

**Time saved:**

- Old workflow: ~2 hours of manual SSH overhead across 10 jobs
- New workflow: 30 seconds to queue + walk away
- **Net savings: ~2 hours**

### Journey 4: Beta 2 Vision - Complete Automation

**User:** Yanis  
**Context:** Weekly benchmark suite - 15 jobs that each produce CSV results  
**Date:** After Beta 2 release (queue + automatic result download)

Yanis has a weekly routine: run 15 optimization benchmarks and analyze results locally in Jupyter.

**Old workflow (pre-SolverPilot):**

1. SSH + rsync + tmux for each job (manual, 2-3 hours)
2. After all jobs complete, SSH back in
3. Manually `scp` each result CSV from server to laptop
4. Organize files locally
5. Start Jupyter analysis

**Total time:** 3-4 hours

**New workflow (Beta 2):**

**Step 1: Configure Result Download**
In Project Settings, Yanis configures result patterns:

- **Output directory pattern:** `results/*.csv`
- **Download location:** `/Users/yanis/Downloads/SolverPilot/{project}/{benchmark}/`
- **Auto-download:** ✓ Enabled

**Step 2: Queue All Benchmarks**
Monday morning:

1. Select all 15 benchmarks
2. Right-click → "Add to Queue"
3. Click "Start Queue"

Queue panel shows:

```
Queue (1 running, 14 pending)
Estimated completion: 11:30 AM (in 3.5 hours)
```

**Step 3: Walk Away**
Yanis closes laptop at 8:00 AM, goes to meetings.

**Background (while Yanis is away):**

- SolverPilot runs each job sequentially
- As each job completes, SolverPilot:
  1. Checks remote `results/` directory
  2. Finds `benchmark_01_results.csv`
  3. Downloads file to `/Users/yanis/Downloads/SolverPilot/gurobi-benchmarks/benchmark_01/benchmark_01_results.csv`
  4. Updates job status to "✓ Completed - Results downloaded (1 file, 2.3 MB)"

**Step 4: Return After Lunch**
Opens SolverPilot at 12:00 PM.

Queue status:

```
Queue (15 completed)
✓ benchmark_01.py - Results downloaded (1 file, 2.3 MB)
✓ benchmark_02.py - Results downloaded (1 file, 1.8 MB)
...
✓ benchmark_15.py - Results downloaded (1 file, 3.1 MB)

Total downloads: 15 files, 34.2 MB
Download location: /Users/yanis/Downloads/SolverPilot/gurobi-benchmarks/
```

**Step 5: Analyze Results Immediately**
Opens Jupyter notebook:

```python
import pandas as pd
from pathlib import Path

results_dir = Path("~/Downloads/SolverPilot/gurobi-benchmarks/").expanduser()
all_results = []

for benchmark_dir in results_dir.iterdir():
    csv_file = next(benchmark_dir.glob("*.csv"))
    df = pd.read_csv(csv_file)
    all_results.append(df)

combined = pd.concat(all_results)
# Start analysis immediately - no manual downloads needed
```

**Aha Moment:**
"I queued 15 jobs at 8 AM. At noon, all results are already on my laptop, organized and ready to analyze. I didn't SSH once. I didn't manually download anything. SolverPilot handled everything from queue → run → download → organize."

**Time saved:**

- Old workflow: 3-4 hours (manual SSH + job management + result downloads)
- New workflow: 30 seconds (queue all jobs, walk away)
- **Net savings: 3-4 hours per week**

**Success Moment:**
Yanis realizes: "I haven't manually SSH'd to the server for Python jobs in two weeks. SolverPilot has completely eliminated that workflow."

---

## Journey Requirements Summary

### From Journey 1 (Current Pain Points):

**Problems to Solve:**

- Manual SSH authentication overhead (10+ times/day)
- Forgetting tmux → lost work on disconnect
- No centralized queue system
- No progress tracking without SSH-ing back
- Manual file transfer (rsync) every time
- Results scattered, manual download (scp)
- No tracking of what ran, what's pending, what failed

### From Journey 2 (Alpha Adoption - Already Implemented):

**Required Capabilities:**

1. **Setup Wizard**
   - Server hostname, username, SSH key configuration
   - Remote base directory path
   - Python version selection
   - Connection testing (validates SSH before saving)

2. **Project Management**
   - Create Python projects with uv
   - Link to local source directory
   - Initialize remote project structure

3. **Benchmark Management**
   - Add Python files as benchmarks
   - Automatic dependency detection via tree-sitter AST parsing
   - Visual list of all benchmarks in project

4. **Dependency Detection & Installation**
   - Parse Python imports (static analysis with tree-sitter)
   - Identify external packages (gurobipy, numpy, etc.)
   - Identify local file dependencies (other .py files)
   - Automatic `uv add <package>` for external dependencies
   - Check existing environment, skip if already installed

5. **Job Execution**
   - One-click "Run" button
   - Automatic rsync of project files to remote
   - SSH connection pooling (bb8) for reusable connections
   - tmux session creation with unique name
   - Start Python script in tmux
   - Return immediately (non-blocking)

6. **Real-Time Monitoring**
   - Stream logs from remote tmux session
   - Parse progress indicators ([x/y] patterns)
   - Show elapsed time (live counter)
   - Detect job completion (exit patterns)
   - Display final status (success/failure)

7. **Resilience**
   - Jobs run in tmux (survive disconnects)
   - Connection pool (avoid repeated SSH handshakes)
   - Local SQLite database (track all jobs)

### From Journey 3 (Beta 1 - Queue System):

**Required Capabilities:**

1. **Multi-Select & Queuing**
   - Select multiple benchmarks (shift-click, ctrl-click)
   - Bulk "Add to Queue" action
   - Queue stored in SQLite (persistent across restarts)

2. **Queue Management**
   - Visual queue panel showing all jobs (pending, running, completed)
   - Start/Pause/Resume queue controls
   - Queue position indicators
   - Estimated completion time (based on average job duration)
   - Job reordering (manual priority adjustment)

3. **Sequential Execution**
   - Single worker process (one job at a time)
   - FIFO ordering (or custom priority)
   - Auto-start next job when current completes
   - Handle failures gracefully (continue to next job)

4. **Queue UI**
   - Real-time status updates for all jobs
   - Current job shows live logs + progress
   - Pending jobs show estimated start time
   - Completed jobs show checkmarks + final status
   - Failed jobs show error indicator + clickable for logs

5. **Job Cancellation**
   - Kill running job (tmux kill-session)
   - Update database status to "cancelled"
   - Automatically start next pending job

6. **Persistence**
   - Queue survives SolverPilot restarts
   - Resume queue processing on app reopen
   - Track partially completed queues

### From Journey 4 (Beta 2 - Result Download):

**Required Capabilities:**

1. **Result Download Configuration**
   - Per-project settings for output patterns
   - Specify output directory on remote (e.g., `results/*.csv`)
   - Configure local download location (project-organized structure)
   - Enable/disable auto-download toggle

2. **Automatic Result Detection**
   - After job completion, check remote output directory
   - Match files against configured patterns (`*.csv`, `*.json`, `*.png`, etc.)
   - Handle various file types and sizes (1KB - 1GB+)

3. **File Transfer**
   - Download files via SSH (using existing connection pool)
   - Show download progress for large files (progress bar)
   - Handle network interruptions (resume partial downloads)
   - Organize locally: `{download-location}/{project}/{benchmark}/`

4. **Download Monitoring**
   - Track download status per job (pending, downloading, complete, failed)
   - Show total downloads (file count, total size)
   - Success/failure notifications (toast messages)
   - Retry mechanism for failed downloads (manual or automatic)

5. **Download History**
   - Track all downloads in database (job_id, file_path, size, timestamp)
   - "Open in Finder/Explorer" quick action
   - "Download location" clickable link in UI

6. **Multi-File Support**
   - Handle jobs that produce multiple output files
   - Download all matching files (not just one)
   - Preserve remote directory structure if configured

**Alternative Implementation Approaches (Decision Deferred):**

- **Approach A:** tree-sitter analysis to predict output files from Python code (e.g., detect `df.to_csv('results.csv')` → download `results.csv`)
- **Approach B:** Folder monitoring (user specifies pattern, download all files in that location after job completes)

## Desktop App Specific Requirements

### Platform Support

**Target Platforms:**

- **Windows** (x64, ARM64 planned)
- **macOS** (Intel, Apple Silicon)
- **Linux** (x64, ARM64)

**Cross-Platform Strategy:**

- Tauri 2 provides consistent API across all platforms
- russh (pure Rust SSH) works identically on all platforms
- SQLite database portable across platforms
- No known platform-specific behavior differences currently

**Platform-Specific Considerations:**

- **File paths:** Cross-platform path handling via Rust's `PathBuf`
- **SSH key locations:** Standard locations (`~/.ssh/`) work across platforms
- **Build artifacts:** Platform-specific installers (.deb, .dmg, .msi, .AppImage)
- **Known risk vectors:**
  - File picker behavior variations across platforms
  - SSH key permission differences (Unix 0600 vs Windows ACLs)
  - Path separator handling (Windows backslashes vs Unix forward slashes)
  - Spaces in file paths (Windows common, Unix less common)

**Testing Strategy (Alpha/Beta):**

- **Primary platform:** Linux (x64) - developer's daily driver
- **CI/CD builds:** All platforms verified for compilation
- **Manual testing:** Linux-only until external user testing begins
- **Known risk:** Platform-specific bugs will surface when shared with Windows/macOS users
- **Mitigation:** Tauri 2 framework abstracts most platform differences
- **Beta exit criteria suggestion:** At least 1 successful external user test per platform before v1.0

**Development Environment:**

- Primary development on Linux
- CI/CD builds and tests on all platforms (GitHub Actions)
- Community testing for platform-specific issues expected during Beta

### System Integration

**Current Integrations (Alpha):**

- **File system access:** File picker dialogs for selecting Python benchmarks
- **SSH key management:** Read SSH keys from standard filesystem locations
- **Local database:** SQLite stored in platform-specific config directory
- **Network:** Outbound SSH connections to remote servers (port 22)

**Planned Integrations:**

**Desktop Notifications (Beta 2):**

- **Scope:** Pairs with automatic result download feature
- **Trigger events:**
  - Job completion (success/failure)
  - Result download complete
  - Queue finished (all jobs processed)
  - Critical errors (SSH connection lost, job failed)
- **Notification content:**
  - Job name and status
  - Execution time
  - Quick action: "Open SolverPilot" or "View Results"
- **Platform implementation:** Tauri notification plugin (cross-platform)
- **User control:** Enable/disable notifications in settings
- **Value proposition:** Reinforces "set and forget" workflow - user doesn't need to keep checking app

**System Tray Icon (Growth Features / v1.0+):**

- **Status indicator:** Running jobs count in tray
- **Quick menu:**
  - Show queue status (X running, Y pending)
  - Open main window
  - Pause/resume queue
  - Quit application
- **Implementation:** Tauri system tray plugin
- **Priority:** Nice-to-have, deferred to v1.0+
- **Note:** Without system tray, users might think closing window = killing jobs. UI should clearly indicate "jobs continue running when window closed"

**OS-Level Permissions:**

- **No special permissions required**
- Standard file picker grants scoped file access
- Outbound network (SSH) allowed by default on all platforms
- SSH keys read from user's home directory (standard filesystem access)
- **No keychain/credential manager integration needed** (SSH keys from filesystem)
- **No firewall rules needed** (outbound SSH on port 22 typically allowed)

### Auto-Update Strategy

**Current State (Alpha/Beta):**

- **Distribution:** GitHub Releases (manual downloads)
- **Update mechanism:** Users manually download new versions
- **Release cadence:** Ad-hoc releases during active development
- **Installers:** Unsigned binaries ("Unverified Publisher" warnings acceptable for Beta)

**Planned Auto-Update (Post-Beta / v1.0+):**

- **In-app updates:** Tauri updater plugin (preferred approach)
- **Update flow:**
  1. SolverPilot checks for updates on startup (configurable interval)
  2. If update available, show in-app notification
  3. User clicks "Update" → background download
  4. Prompt to restart and apply update
  5. Update applied seamlessly
- **Update channel options:**
  - Stable (default): Production releases only
  - Beta (opt-in): Early access to Beta features
- **Implementation requirements:**
  - **Code signing for security** (macOS, Windows)
    - Apple Developer certificate ($99/year)
    - Windows code signing certificate ($200-400/year)
    - CI/CD signing infrastructure setup
  - Update manifest endpoint (GitHub Releases or custom server)
  - Delta updates for efficiency (download only changed files)
- **User control:**
  - Auto-check for updates (on/off)
  - Auto-download updates (on/off)
  - Manual "Check for Updates" button

**Priority:** Post-Beta (v1.0+), not blocking for Beta releases

**Cost Consideration:** Auto-update implementation deferred until budget available for code signing certificates (~$300-500/year total).

### Offline Capabilities

**Offline Mode Support:**

**What Works Offline:**

- ✅ **View job history:** Browse past jobs, view logs, execution times
- ✅ **View project configuration:** See projects, benchmarks, dependencies
- ✅ **Queue jobs offline:** Select benchmarks and add to queue (stored locally in SQLite)
- ✅ **UI navigation:** Full UI remains functional
- ✅ **Local data management:** Edit project settings, remove benchmarks

**What Requires Network:**

- ❌ **Run benchmarks:** SSH connection to remote server required
- ❌ **Download results:** Requires network to retrieve files from server
- ❌ **Dependency installation:** tree-sitter parsing works offline, but `uv` package operations require network
- ❌ **SSH connection testing:** Configuration wizard validation requires network

**Offline Queue Behavior:**

- **Queueing offline:** User can select benchmarks and queue jobs without network
- **Queue persistence:** Queued jobs stored in SQLite, survive app restarts
- **Execution on reconnect:** When network returns, user manually starts queue
- **No automatic sync:** SolverPilot does not auto-retry queue when network detected (user initiates)

**Dependency Re-Validation (Beta 1 Feature):**

- **Problem:** Python files may change between queue time and execution time
- **Solution:** Re-run tree-sitter dependency analysis when starting any queued job
- **Behavior:**
  1. User queues `benchmark.py` (imports: numpy, pandas)
  2. Later, user edits `benchmark.py` → adds `import scipy`
  3. User clicks "Start Queue"
  4. SolverPilot detects new dependency (`scipy`), runs `uv add scipy`, then executes
- **Performance:** Adds ~1-2 seconds per job startup (tree-sitter parsing + uv check)
- **Value:** Prevents "ModuleNotFoundError" from stale queue entries
- **Design philosophy:** Queued jobs reflect _current state_ of Python file when executed, not when queued

**Network State Handling:**

- **Connection loss detection:** SSH operations fail gracefully with error messages
- **User feedback:** Toast notifications for network errors ("SSH connection failed - check network")
- **Recovery:** User manually retries once network is restored

**Acceptable Constraints:**

- **Network required for core workflow:** "Requires active network connection to run jobs" is acceptable
- **No offline execution:** Remote server execution inherently requires network
- **Manual reconnect:** User responsible for resuming queue after network outage

### Implementation Considerations

**Desktop-Specific Technical Decisions:**

1. **Tauri 2 Architecture:**
   - IPC communication between Rust backend and Svelte frontend
   - Native desktop performance (not Electron)
   - Smaller binary size (~15-20 MB installers)

2. **Single-Window Application:**
   - Main window with 3-panel layout (not multi-window)
   - Modal dialogs for setup wizard and configuration
   - System tray provides quick access (future enhancement)
   - **UI clarity needed:** Indicate "jobs continue running when window closed" (since no system tray in Beta)

3. **Background Processing:**
   - SSH operations run asynchronously (Tokio runtime)
   - UI remains responsive during long-running jobs
   - Polling architecture (2-second intervals) for job status updates

4. **Data Persistence:**
   - SQLite database in platform-specific config directory
   - Config file (TOML) for server settings
   - No cloud sync (local-first architecture)

5. **Security Considerations:**
   - SSH keys never exposed to frontend
   - Credentials handled in Rust backend only
   - No telemetry or usage tracking

6. **Installation & Distribution:**
   - Platform-specific installers (no universal binary)
   - Unsigned binaries for Alpha/Beta ("Unverified Publisher" warnings acceptable)
   - Code signing deferred to v1.0+ for production releases

**Platform Testing Recommendations:**

- **Test scenario for Beta 1:** "Queue job offline, modify dependencies, execute online"
  1. Queue benchmark.py (imports numpy)
  2. Edit benchmark.py → add `import scipy`
  3. Start queue
  4. **Expected:** SolverPilot detects new dependency, installs scipy, runs successfully
  5. **Failure mode:** Job fails with "ModuleNotFoundError: scipy"

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-Solving MVP with Dogfooding Focus

**Philosophy:**
SolverPilot is built as a personal productivity tool to solve the developer's (Yanis) own SSH workflow pain. The phased approach validates each capability through real usage before adding the next layer of automation.

**Resource Requirements:**

- **Team size:** Solo developer (Yanis)
- **Skills required:** Rust, Svelte 5, Tauri 2, SSH/networking, Python ecosystem
- **Development model:** Dogfooding - developer is the primary user
- **Timeline:** Alpha working → Beta 1 (queue) → Beta 2 (downloads) → Growth features as needed

**Success Definition:**
Developer achieves "never SSH manually to server for Python job execution" when Beta 2 is complete.

**No External User Pressure:**
Growth features are genuine nice-to-haves. No colleagues waiting to use SolverPilot, so timeline is flexible and feature expansion happens when developer needs it, not due to external demands.

---

### Phase 1: Alpha (Current - Complete ✅)

**Status:** Working and validated

**Core User Journey Supported:** Journey 2 - First Liberation (setup wizard → run single job → monitor progress)

**Must-Have Capabilities (Implemented):**

1. **Setup Wizard:** One-time server configuration (hostname, SSH key, remote paths)
2. **Project Management:** Create Python projects with uv
3. **Benchmark Management:** Add Python files, automatic dependency detection via tree-sitter
4. **Dependency Installation:** Automatic `uv add` for external packages
5. **Job Execution:** One-click run with automatic rsync + tmux + SSH connection pooling
6. **Real-Time Monitoring:** Live logs, progress parsing ([x/y] patterns), elapsed time
7. **Resilience:** tmux persistence (jobs survive disconnects), connection pooling (bb8)
8. **Local Database:** SQLite tracking of projects, benchmarks, jobs

**Alpha Exit Criteria Met:**

- Core automation working (SSH pooling, dependency detection, tmux, monitoring)
- Single job workflow validated through dogfooding
- No manual SSH required for individual job execution

---

### Phase 2: Beta 1 (Queue System - Next Priority 🎯)

**Target User Journey:** Journey 3 - Queue Changes Everything (select 10 benchmarks → queue all → walk away → return to completed jobs)

**Must-Have Capabilities:**

**1. Job Queue Management:**

- Select multiple Python files for execution
- Queue jobs in database (FIFO ordering)
- Single worker process (sequential execution, one job at a time)
- Queue status tracking (pending, running, completed, failed)
- Start/pause/resume queue controls
- Job cancellation (kill running job via tmux)

**2. Queue UI:**

- Visual queue display (list of pending/running/completed jobs)
- Queue position indicators
- Estimated time remaining (based on average job duration)
- Manual job reordering (priority adjustment)

**3. Dependency Re-Validation (Desktop App Requirement):**

- Re-run tree-sitter analysis when starting any queued job
- Handle Python file changes between queue time and execution time
- Auto-install new dependencies discovered during re-validation
- Prevent "ModuleNotFoundError" from stale queue entries

**4. Offline Queue Support:**

- Queue jobs without network connection
- Jobs persist in SQLite across app restarts
- Execute when network available (user-initiated)

**Beta 1 Exit Criteria:**

- User successfully queues and runs 50+ jobs without manual SSH
- Queue system handles job sequencing correctly
- SolverPilot reliability: 100% (zero failures in SSH, sync, queue management)
- Job success rate: Measured and reported (target: >90% for well-tested scripts)

**Success Moment:** "Queue 10 jobs, go to lunch, return to 9 completed + 1 failed with clear error log"

---

### Phase 3: Beta 2 (Automatic Result Download 🎯)

**Target User Journey:** Journey 4 - Complete Automation (queue 15 jobs → all complete with results downloaded → analyze locally in Jupyter)

**Must-Have Capabilities:**

**1. Automatic Result Download:**

- Detect or configure output files created by Python scripts
- Download results to local machine after job completion
- Organize downloads by project/job/benchmark
- Handle various file sizes (1KB to 1GB+)
- Handle network interruptions gracefully (resume partial downloads)

**2. Result Download Configuration:**

- User specifies output file patterns (e.g., `results/*.csv`, `output/*.json`)
- OR tree-sitter detection of file write operations (exploratory approach)
- File size limits (warn on >100MB downloads)
- Storage location configuration

**3. Download Monitoring:**

- Progress indicators for large file downloads
- Success/failure notifications
- Retry mechanism for failed downloads

**4. Desktop Notifications (Desktop App Requirement):**

- Job completion notifications
- Result download complete notifications
- Queue finished notifications
- Critical error notifications
- Platform implementation via Tauri notification plugin

**Beta 2 Exit Criteria:**

- 10+ jobs with successful result downloads
- Files retrieved match expected outputs
- Download success rate >95% (accounting for network issues)
- User workflow: Queue → Run → Auto-download → Review results locally
- **Primary success achieved:** "Never SSH manually to server for Python job execution"

**Implementation Decision Deferred:** Choice between tree-sitter output detection vs folder monitoring approach will be made during Beta 2 planning.

---

### Phase 4: Growth Features (Post-Beta / v1.0+)

**Timing:** Implemented as needed by developer, no external pressure

**Failure Recovery & Resilience:**

- Automatic retry on SSH connection failures
- File sync retry with exponential backoff
- Result download resume on network interruption
- Queue recovery after SolverPilot crash

**Performance Optimization Track:**

- Benchmark SSH connection overhead vs manual
- Optimize dependency detection speed
- Profile and optimize file sync performance
- Connection pool tuning

**Enhanced Monitoring:**

- Job history visualization
- Execution time trends
- Success/failure analytics
- Server resource monitoring (CPU, memory, disk usage)

**Testing & Quality:**

- Automated test suite for core workflows
- Synthetic test jobs for edge case validation
- CI/CD integration for regression testing

**Desktop Enhancements:**

- System tray icon with quick status
- Advanced notification controls
- Multi-project management improvements

**Multi-User Support (If Needed):**

- Shared server configurations
- Team job history
- Collaborative project management

**Note:** Growth features are genuine nice-to-haves. No colleagues currently waiting to use SolverPilot, so these are implemented when developer needs them, not on external timeline.

---

### Phase 5: Vision (v2.0 - Future Exploration)

**Timing:** Long-term possibilities, may or may not be implemented

**Advanced Visualization:**

- Graph optimization results (gap tracking over iterations)
- Render result images directly in app
- Interactive result exploration
- Comparative analysis across job runs

**Enhanced Automation:**

- Parallel job execution (multiple tmux sessions)
- Conditional job chains (run B after A completes)
- Scheduled job execution
- Smart retry on transient failures

**Ecosystem Integration:**

- Jupyter notebook integration
- CI/CD pipeline integration
- Cloud provider support (AWS, Azure, GCP)
- Docker container execution support

**Platform Expansion:**

- Auto-update via Tauri updater plugin (requires code signing investment)
- Broader platform testing and validation
- Community-driven feature requests

---

### Risk Mitigation Strategy

**Technical Risks:**

**Risk 1: SSH Connection Reliability**

- **Mitigation (Implemented):** Connection pooling via bb8, persistent connections, graceful error handling
- **Fallback:** User manually retries on connection failure (acceptable for personal tool)

**Risk 2: Dependency Detection Accuracy**

- **Mitigation (Implemented):** tree-sitter AST parsing (100% for static imports)
- **Known limitations:** Dynamic imports, exec/eval statements (documented, acceptable)
- **Fallback:** User manually adds missing packages to pyproject.toml

**Risk 3: Cross-Platform Compatibility**

- **Mitigation:** Tauri 2 framework abstracts platform differences
- **Testing strategy:** Linux-only for Alpha/Beta, community testing for Win/Mac
- **Risk acceptance:** Platform-specific bugs expected when sharing with external users
- **Deferred:** Full multi-platform testing to v1.0 or when external users appear

**Risk 4: Result Download Complexity (Beta 2)**

- **Mitigation:** Two implementation approaches under consideration (tree-sitter vs folder monitoring)
- **Decision deferred:** Choose approach during Beta 2 planning based on implementation simplicity
- **Fallback:** If auto-detection fails, manual pattern configuration always available

**Market Risks:**

**Risk 1: Limited User Base**

- **Reality:** Built as personal tool first, no external users currently
- **Mitigation:** Not a risk - no market pressure, flexible timeline
- **Opportunity:** If shared with 3-5 colleagues and adopted, validates broader potential

**Risk 2: Competing with Manual SSH**

- **Mitigation:** Dogfooding proves value (5-10 min/job time savings)
- **Success metric:** Developer stops using manual SSH workflows
- **Fallback:** Not applicable - building for personal use

**Resource Risks:**

**Risk 1: Solo Developer Bandwidth**

- **Mitigation:** Phased approach allows incremental progress
- **Scope flexibility:** No external deadlines, features added when needed
- **Pragmatic scoping:** Beta 1 and Beta 2 are lean, focused scopes

**Risk 2: Time Investment vs Value**

- **Mitigation:** Already seeing Alpha value (core automation working)
- **Validation:** Each phase validates through dogfooding before next phase
- **Exit strategy:** Alpha is useful standalone; Beta 1 adds queue; Beta 2 completes vision

## Functional Requirements

_Generated through comprehensive discovery: journey analysis + 5 Advanced Elicitation methods (5 Whys, Pre-mortem, User Persona, Devil's Advocate, Failure Mode Analysis)_

**Total: 216 Requirements across 8 Capability Areas**

### Configuration & Setup (38 FRs)

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

### Project Management (13 FRs)

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

### Benchmark Management (16 FRs)

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

### Dependency Management (18 FRs)

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

### Job Execution & Experiment Management (24 FRs)

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

### Job Monitoring & Status (20 FRs)

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

### Queue Management (Beta 1) (36 FRs)

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

### Result Management (Beta 2) (51 FRs)

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

---

**Requirements Summary:**

- **Configuration & Setup:** 38 FRs (FR1-FR38)
- **Project Management:** 13 FRs (FR39-FR54)
- **Benchmark Management:** 16 FRs (FR55-FR72)
- **Dependency Management:** 18 FRs (FR73-FR94)
- **Job Execution & Experiment Management:** 24 FRs (FR95-FR121)
- **Job Monitoring & Status:** 20 FRs (FR122-FR147)
- **Queue Management (Beta 1):** 36 FRs (FR148-FR191)
- **Result Management (Beta 2):** 51 FRs (FR192-FR252)

**Total Functional Requirements:** 216

**Coverage Validation:**
✅ All journey capabilities covered (Manual SSH → Alpha → Beta 1 → Beta 2)
✅ Edge cases from 5 Whys Deep Dive included (SSH passphrases, re-validation, active indicators)
✅ Failure scenarios from Pre-mortem Analysis included (crashes, silent failures, runaway jobs)
✅ Real-world workflow needs from User Persona included (arguments, multi-server, custom parsing)
✅ Ambiguities from Devil's Advocate included (server switching, conflicts, notification grouping)
✅ Robustness from Failure Mode Analysis included (proxy, 2FA, tmux cleanup, reboot handling)

## Non-Functional Requirements

_Validated through Party Mode technical review (Winston, Murat, Amelia)_

### Performance

**SSH Connection Performance:**

- **NFR-P1:** SSH connection establishment must complete within 5 seconds under normal network conditions (<50ms latency) _(Target, not strict requirement for high-latency networks)_
- **NFR-P2:** Connection pool reuse must reduce subsequent SSH operations to <500ms (eliminating handshake overhead)
- **NFR-P3:** File sync operations (rsync) must provide real-time progress feedback for transfers >10MB

**UI Responsiveness:**

- **NFR-P4:** All UI interactions must respond within 100ms (button clicks, navigation, input)
- **NFR-P5:** Background SSH operations must not block UI thread (async execution mandatory via Tokio runtime)
- **NFR-P6:** Job status polling must update UI within 2 seconds of remote state change _(Based on default 2-second polling interval)_

**Dependency Detection Performance:**

- **NFR-P7:** tree-sitter AST parsing must complete within 2 seconds for Python files <5000 lines (average case, worst-case benchmarks required for complex imports)
- **NFR-P8:** Dependency analysis must handle recursive imports up to 10 levels deep

**Performance Regression Detection:**

- **NFR-P9:** CI/CD pipeline must include performance benchmarks for SSH connection pooling and dependency analysis with <10% regression tolerance between releases

**Philosophy:**
Sufficient performance for single-user desktop workflow. Performance optimization is deferred to Growth phase (v1.0+) unless it impacts usability.

### Reliability

**SolverPilot Core Reliability Target:**

- **NFR-R1:** SolverPilot must have zero known bugs in core workflows (SSH connection, file sync, job submission, queue management, monitoring) at time of release _(100% reliability goal validated through testing)_
- **NFR-R2:** Job submission must succeed 100% when server is reachable and credentials are valid
- **NFR-R3:** Connection pool must gracefully handle transient network issues with retry logic (3 retries with exponential backoff) before surfacing error to user
- **NFR-R4:** Queue persistence must survive application crashes without data loss (SQLite ACID properties)

**Failure Boundaries:**

- **NFR-R5:** SolverPilot failures must be clearly distinguished from Python script failures, server issues, or network problems
- **NFR-R6:** All error messages must be actionable (tell user what went wrong and how to fix it)

**Data Integrity:**

- **NFR-R7:** SQLite database operations must complete atomically (no partial writes on failure, verified via crash injection tests)
- **NFR-R8:** Job status tracking must accurately reflect remote job state (no phantom "running" jobs)
- **NFR-R9:** File sync must detect obvious transfer failures (size mismatches, connection drops, non-zero rsync exit codes)

**Resilience:**

- **NFR-R10:** tmux sessions must survive SSH disconnections without job termination
- **NFR-R11:** Application must reconnect to existing tmux sessions after network interruption. If tmux session not found, mark job as "unknown state" and prompt user for manual verification
- **NFR-R12:** Queue processing must resume correctly after application restart

### Security

**Credential Protection:**

- **NFR-S1:** SSH private keys must never be exposed to frontend (Rust backend only)
- **NFR-S2:** SSH key passphrases must be zeroized in memory immediately after use
- **NFR-S3:** Configuration file must use restrictive file permissions (0600 on Unix, ACL-protected on Windows)
- **NFR-S4:** No credentials may be logged to console or debug output

**Local Data Security:**

- **NFR-S5:** SQLite database must be stored in platform-specific user config directory (not world-readable)
- **NFR-S6:** Connection configuration must not include passwords (SSH key authentication only)
- **NFR-S7:** Sensitive data must not be transmitted to any external service (100% local-first architecture)

**Input Validation:**

- **NFR-S8:** All user-provided paths must be sanitized to prevent directory traversal attacks
- **NFR-S9:** SSH command arguments must be escaped to prevent command injection (using Rust `std::process::Command` with separate args)
- **NFR-S10:** SQL queries must use parameterization to prevent SQL injection (SQLx compile-time checking enforced)

**Network Security:**

- **NFR-S11:** SSH connections must verify host keys (prevent man-in-the-middle attacks)
- **NFR-S12:** SSH protocol must use secure algorithms (Ed25519/RSA keys, no legacy DSA)

### Usability

**Setup Experience:**

- **NFR-U1:** Setup wizard must complete in <5 minutes for users with pre-existing SSH key pairs and basic SSH knowledge
- **NFR-U2:** Connection test must provide clear success/failure feedback with specific error messages
- **NFR-U3:** First-run experience must guide user through server configuration with validation at each step

**Error Handling & Feedback:**

- **NFR-U4:** All error messages must be user-friendly (no raw technical stack traces shown)
- **NFR-U5:** Error messages must suggest corrective actions ("Check network connection", "Verify SSH key path")
- **NFR-U6:** Toast notifications must auto-dismiss after 5-8 seconds for non-critical messages
- **NFR-U7:** Critical errors must persist until user acknowledgment

**Progress Visibility:**

- **NFR-U8:** Long-running operations (>3 seconds) must show progress indicators
- **NFR-U9:** Job monitoring must update at minimum every 2 seconds during execution
- **NFR-U10:** Queue must show estimated time remaining based on job history

**Learnability:**

- **NFR-U11:** UI must follow platform conventions (Windows/macOS/Linux native patterns)
- **NFR-U12:** Primary workflows must be discoverable without documentation (intuitive controls)
- **NFR-U13:** Tooltips or help text must explain non-obvious features on hover

**Platform Consistency:**

- **NFR-U14:** Application must behave identically across Windows, macOS, and Linux (no platform-specific feature gaps)
- **NFR-U15:** File path handling must correctly process spaces and unicode characters. Emoji and non-UTF-8 filenames may be unsupported with clear error messages when detected

**Network State Handling:**

- **NFR-U16:** Application must clearly indicate network connectivity status and disable/grey-out network-dependent actions when offline

**Cross-Platform Filesystem Compatibility:**

- **NFR-U17:** Application must handle platform-specific filesystem differences (case sensitivity on Linux vs Windows/macOS, Windows long path limitations, special characters) with clear error messages when unsupported scenarios detected

### Maintainability

**Code Quality:**

- **NFR-M1:** Rust code must pass clippy pedantic linting with zero warnings
- **NFR-M2:** No `unwrap()` or `expect()` in production code (explicit error handling required)
- **NFR-M3:** TypeScript strict mode must be enabled with zero `any` types
- **NFR-M4:** All public APIs must have inline documentation

**Build & Deployment:**

- **NFR-M5:** CI/CD pipeline must build successfully for all supported platforms
- **NFR-M6:** Production builds must complete within 20 minutes in CI/CD
- **NFR-M7:** Installer artifacts must be <25MB per platform

**Dependency Management:**

- **NFR-M8:** Security audits (cargo-deny) must pass with zero known vulnerabilities
- **NFR-M9:** Dependencies must be locked (Cargo.lock, bun.lockb) for reproducible builds

**Testing & Regression Prevention:**

- **NFR-M10:** Core workflows (create project, add benchmark, queue jobs, monitor execution) must have automated integration tests that run in CI/CD

**Error Logging & Debugging:**

- **NFR-M11:** Application must log detailed error context (stack traces, SSH debug output, rsync exit codes) to platform-specific log directory for debugging

**Data Migration & Upgrades:**

- **NFR-M12:** Application must detect and migrate data from previous versions (SQLite schema, config format) without data loss during upgrades

---

**NFR Summary (Updated):**

- **Performance:** 9 requirements (sufficient speed + regression detection)
- **Reliability:** 12 requirements (zero known bugs target)
- **Security:** 12 requirements (credential protection, local-first security)
- **Usability:** 17 requirements (intuitive desktop UX + offline handling + cross-platform)
- **Maintainability:** 12 requirements (code quality, testing, logging, upgrades)

**Total Non-Functional Requirements:** 62 (up from 56)

**Key Improvements from Party Mode Review:**

- ✅ Network-timing NFRs softened to targets (realistic for variable latency)
- ✅ 100% reliability rephrased as "zero known bugs" (testable)
- ✅ Offline UX behavior explicitly defined
- ✅ Cross-platform filesystem edge cases addressed
- ✅ Regression testing and performance benchmarking added
- ✅ Error logging for debugging included
- ✅ Data migration for upgrades planned
- ✅ Failure modes documented for complex scenarios (tmux reconnection)

**Skipped Categories (Deliberately):**

- ❌ Scalability (not relevant for solo-user desktop tool)
- ❌ Public Accessibility/WCAG (developer tool, not public product)
- ❌ Compliance (no regulatory requirements for personal tool)
