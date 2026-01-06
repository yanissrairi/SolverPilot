import { invoke } from '@tauri-apps/api/core';
import type {
  AppConfig,
  Benchmark,
  Job,
  SyncStatus,
  SshKeyStatus,
  JobStatusResponse,
} from './types';

// Config
export async function loadConfig(): Promise<AppConfig> {
  return invoke('load_config');
}

// SSH
export async function initSsh(): Promise<string> {
  return invoke('init_ssh');
}

export async function closeSsh(): Promise<void> {
  return invoke('close_ssh');
}

export async function testSsh(): Promise<boolean> {
  return invoke('test_ssh');
}

export async function checkSshKeyStatus(): Promise<SshKeyStatus> {
  return invoke('check_ssh_key_status');
}

export async function addSshKey(passphrase: string): Promise<void> {
  return invoke('add_ssh_key', { passphrase });
}

// Sync
export async function checkSyncStatus(): Promise<SyncStatus> {
  return invoke('check_sync_status');
}

export async function syncCode(): Promise<void> {
  return invoke('sync_code');
}

// Benchmarks
export async function scanBenchmarks(): Promise<Benchmark[]> {
  return invoke('scan_benchmarks');
}

// Jobs
export async function queueJobs(benchmarkNames: string[]): Promise<Job[]> {
  return invoke('queue_jobs', { benchmarkNames });
}

export async function startNextJob(): Promise<Job | null> {
  return invoke('start_next_job');
}

export async function stopJob(): Promise<void> {
  return invoke('stop_job');
}

export async function killJob(): Promise<void> {
  return invoke('kill_job');
}

export async function getJobLogs(lines: number): Promise<string> {
  return invoke('get_job_logs', { lines });
}

export async function getJobStatus(): Promise<JobStatusResponse> {
  return invoke('get_job_status');
}

// History
export async function loadHistory(limit: number): Promise<Job[]> {
  return invoke('load_history', { limit });
}

export async function deleteJob(jobId: number): Promise<void> {
  return invoke('delete_job', { jobId });
}
