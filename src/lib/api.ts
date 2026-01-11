import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AppConfig,
  Benchmark,
  Project,
  Job,
  SyncStatus,
  SshKeyStatus,
  JobStatusResponse,
  DependencyAnalysis,
} from './types';

// =============================================================================
// Config
// =============================================================================

/**
 * Vérifie si le fichier de configuration existe
 */
export async function checkConfigExists(): Promise<boolean> {
  return invoke('check_config_exists');
}

/**
 * Retourne le chemin du fichier de configuration
 */
export async function getConfigPath(): Promise<string> {
  return invoke('get_config_path');
}

/**
 * Sauvegarde la configuration
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config });
}

/**
 * Charge la configuration depuis le fichier
 */
export async function loadConfig(): Promise<AppConfig> {
  return invoke('load_config');
}

// =============================================================================
// SSH
// =============================================================================

export async function initSsh(): Promise<string> {
  return invoke('init_ssh');
}

export async function closeSsh(): Promise<void> {
  return invoke('close_ssh');
}

export async function testSsh(): Promise<boolean> {
  return invoke('test_ssh');
}

/**
 * Test SSH direct (pour le wizard de setup)
 */
export async function testSshDirect(passphrase?: string): Promise<void> {
  return invoke('test_ssh_direct', { passphrase: passphrase ?? null });
}

export async function checkSshKeyStatus(): Promise<SshKeyStatus> {
  return invoke('check_ssh_key_status');
}

export async function addSshKey(passphrase: string): Promise<void> {
  return invoke('add_ssh_key', { passphrase });
}

// =============================================================================
// Sync
// =============================================================================

export async function checkSyncStatus(): Promise<SyncStatus> {
  return invoke('check_sync_status');
}

export async function syncCode(): Promise<void> {
  return invoke('sync_code');
}

/**
 * Synchronise uniquement les fichiers de dépendances d'un benchmark
 * @returns Le nombre de fichiers synchronisés
 */
export async function syncBenchmarkDeps(benchmarkPath: string): Promise<number> {
  return invoke('sync_benchmark_deps', { benchmarkPath });
}

// =============================================================================
// Projects
// =============================================================================

/**
 * Liste tous les projets
 */
export async function listProjects(): Promise<Project[]> {
  return invoke('list_projects');
}

/**
 * Crée un nouveau projet avec un environnement Python via uv
 */
export async function createProject(name: string, pythonVersion: string): Promise<Project> {
  return invoke('create_project', { name, pythonVersion });
}

/**
 * Supprime un projet et son dossier
 */
export async function deleteProject(projectId: number): Promise<void> {
  return invoke('delete_project', { projectId });
}

/**
 * Définit le projet actif
 */
export async function setActiveProject(projectId: number): Promise<Project> {
  return invoke('set_active_project', { projectId });
}

/**
 * Récupère le projet actif (null si aucun)
 */
export async function getActiveProject(): Promise<Project | null> {
  return invoke('get_active_project');
}

// =============================================================================
// Python Version Management
// =============================================================================

/**
 * Liste les versions Python disponibles via uv
 */
export async function listPythonVersions(): Promise<string[]> {
  return invoke('list_python_versions');
}

/**
 * Change la version Python du projet actif
 */
export async function setProjectPythonVersion(version: string): Promise<void> {
  return invoke('set_project_python_version', { version });
}

// =============================================================================
// Project Benchmarks
// =============================================================================

/**
 * Ouvre le dialog pour sélectionner un fichier Python
 * @returns Le chemin du fichier sélectionné ou null si annulé
 */
export async function pickBenchmarkFile(): Promise<string | null> {
  const selected = await open({
    title: 'Sélectionner un benchmark Python',
    multiple: false,
    filters: [{ name: 'Python', extensions: ['py'] }],
  });
  return selected;
}

/**
 * Ajoute un benchmark au projet actif
 */
export async function addBenchmarkToProject(filePath: string): Promise<Benchmark> {
  return invoke('add_benchmark_to_project', { filePath });
}

/**
 * Supprime un benchmark du projet
 */
export async function removeBenchmarkFromProject(benchmarkId: number): Promise<void> {
  return invoke('remove_benchmark_from_project', { benchmarkId });
}

/**
 * Liste les benchmarks du projet actif
 */
export async function listProjectBenchmarks(): Promise<Benchmark[]> {
  return invoke('list_project_benchmarks');
}

/**
 * Analyse les dépendances Python d'un fichier benchmark
 */
export async function getBenchmarkDependencies(benchmarkPath: string): Promise<DependencyAnalysis> {
  return invoke('get_benchmark_dependencies', { benchmarkPath });
}

// =============================================================================
// Project Dependencies
// =============================================================================

/**
 * Ajoute une dépendance au projet actif via `uv add`
 */
export async function addProjectDependency(packageName: string): Promise<string> {
  return invoke('add_project_dependency', { packageName });
}

/**
 * Supprime une dépendance du projet actif via `uv remove`
 */
export async function removeProjectDependency(packageName: string): Promise<string> {
  return invoke('remove_project_dependency', { packageName });
}

/**
 * Met à jour toutes les dépendances du projet actif
 */
export async function updateProjectDependencies(): Promise<string> {
  return invoke('update_project_dependencies');
}

/**
 * Liste les dépendances du projet actif (depuis pyproject.toml)
 */
export async function listProjectDependencies(): Promise<string[]> {
  return invoke('list_project_dependencies');
}

/**
 * Synchronise l'environnement du projet actif via `uv sync`
 */
export async function syncProjectEnvironment(): Promise<string> {
  return invoke('sync_project_environment');
}

// =============================================================================
// Jobs
// =============================================================================

export async function queueJobs(benchmarkNames: string[]): Promise<Job[]> {
  return invoke('queue_jobs', { benchmarkNames });
}

/**
 * Queue benchmarks by their IDs with queue position tracking (Story 1.2)
 */
export async function queueBenchmarks(benchmarkIds: number[]): Promise<Job[]> {
  return invoke('queue_benchmarks', { benchmarkIds });
}

/**
 * Get all queued jobs ordered by status priority (Story 1.3)
 * Returns jobs sorted: running → pending → completed/failed → killed
 */
export async function getAllQueueJobs(): Promise<Job[]> {
  return invoke('get_all_queue_jobs');
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

// =============================================================================
// History
// =============================================================================

export async function loadHistory(limit: number): Promise<Job[]> {
  return invoke('load_history', { limit });
}

export async function deleteJob(jobId: number): Promise<void> {
  return invoke('delete_job', { jobId });
}
