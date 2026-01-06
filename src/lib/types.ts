export interface AppConfig {
  ssh: {
    host: string;
    user: string;
    use_agent: boolean;
  };
  paths: {
    local_code: string;
    local_benchmarks: string;
    local_results: string;
    remote_base: string;
  };
  polling: {
    interval_seconds: number;
  };
}

export interface Benchmark {
  name: string;
  path: string;
}

export type JobStatus = 'pending' | 'running' | 'completed' | 'failed' | 'killed';

export interface Job {
  id: number;
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

export interface JobStatusResponse {
  job: Job | null;
  logs: string;
  progress: number;
  progress_text: string;
  elapsed_seconds: number;
  is_finished: boolean;
  error: string | null;
}
