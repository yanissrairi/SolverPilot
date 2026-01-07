// =============================================================================
// Configuration
// =============================================================================

export interface AppConfig {
  ssh: {
    host: string;
    user: string;
    use_agent: boolean;
    key_path: string;
  };
  remote: {
    remote_base: string;
  };
  polling: {
    interval_seconds: number;
  };
  gurobi: {
    home: string;
    license_file: string;
  };
  tools: {
    uv_path: string;
  };
}

// =============================================================================
// Projects
// =============================================================================

export interface Project {
  id: number;
  name: string;
  python_version: string;
  created_at: string;
  updated_at: string;
}

// =============================================================================
// Benchmarks
// =============================================================================

export interface Benchmark {
  id: number;
  project_id: number;
  name: string;
  path: string;
  created_at: string;
}

// =============================================================================
// Jobs
// =============================================================================

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'killed';

export interface Job {
  id: number;
  project_id: number | null;
  benchmark_name: string;
  status: JobStatus;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
  progress_current: number;
  progress_total: number;
  results_path: string | null;
  error_message: string | null;
  log_content: string;
}

export interface JobStatusResponse {
  job: Job | null;
  logs: string;
  progress: number;
  progress_text: string;
  elapsed_seconds: number;
  is_finished: boolean;
  error: string | null;
}

// =============================================================================
// SSH & Sync
// =============================================================================

export type SyncStatus =
  | { type: 'Checking' }
  | { type: 'UpToDate' }
  | { type: 'Modified'; data: { count: number; files: string[] } }
  | { type: 'Syncing' }
  | { type: 'Error'; data: { message: string } };

export type SshKeyStatus =
  | { type: 'InAgent' }
  | { type: 'NeedsPassphrase'; data: { key_path: string } }
  | { type: 'NoKey'; data: { expected_path: string } }
  | { type: 'NoAgent' };

// =============================================================================
// Dependency Analysis
// =============================================================================

export interface LocalDependency {
  module_name: string;
  file_path: string;
  exists: boolean;
  children: LocalDependency[];
}

export interface ExternalPackage {
  name: string;
  in_pyproject: boolean;
}

export interface DependencyAnalysis {
  root: string;
  local_files: LocalDependency[];
  external_packages: ExternalPackage[];
}
