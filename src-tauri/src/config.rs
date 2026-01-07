use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub ssh: SshConfig,
    pub remote: RemoteConfig,
    pub polling: PollingConfig,
    #[serde(default)]
    pub gurobi: GurobiConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SshConfig {
    pub host: String,
    pub user: String,
    pub use_agent: bool,
    /// Chemin de la clé SSH (optionnel, défaut: `~/.ssh/id_rsa`)
    #[serde(default = "default_key_path")]
    pub key_path: String,
}

fn default_key_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    format!("{home}/.ssh/id_rsa")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteConfig {
    /// Dossier de base sur le serveur (partagé entre tous les projets)
    pub remote_base: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollingConfig {
    pub interval_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GurobiConfig {
    /// Gurobi home directory (e.g., ~/gurobi1300/linux64)
    #[serde(default)]
    pub home: String,
    /// Path to Gurobi license file
    #[serde(default)]
    pub license_file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolsConfig {
    /// Path to uv command
    #[serde(default = "default_uv_path")]
    pub uv_path: String,
}

fn default_uv_path() -> String {
    "~/.local/bin/uv".to_string()
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            uv_path: default_uv_path(),
        }
    }
}

impl AppConfig {
    /// Retourne le chemin racine du projet (parent de src-tauri/)
    pub fn project_root() -> PathBuf {
        // CARGO_MANIFEST_DIR = chemin vers src-tauri/ à la compilation
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // Remonter d'un niveau pour avoir la racine du projet
        manifest_dir.parent().unwrap_or(&manifest_dir).to_path_buf()
    }

    /// Charge la configuration depuis `<project_root>/config.toml`
    /// Utilise zero-copy parsing (toml 0.9+) pour moins d'allocations mémoire
    pub fn load() -> Result<Self, String> {
        let config_path = Self::project_root().join("config.toml");

        let bytes = std::fs::read(&config_path)
            .map_err(|e| format!("Erreur lecture {}: {e}", config_path.display()))?;

        toml::de::from_slice(&bytes)
            .map_err(|e| format!("Erreur parsing {}: {e}", config_path.display()))
    }

    /// Chemin d'un projet spécifique sur le serveur
    pub fn remote_project_path(&self, project_name: &str) -> String {
        format!("{}/projects/{}", self.remote.remote_base, project_name)
    }

    /// Chemin du code d'un projet sur le serveur
    pub fn remote_project_code_path(&self, project_name: &str) -> String {
        format!("{}/projects/{}/code", self.remote.remote_base, project_name)
    }

    /// Chemin du code sur le serveur (deprecated - use `remote_project_code_path`)
    #[deprecated(note = "Use remote_project_code_path instead for multi-project support")]
    pub fn remote_code_path(&self) -> String {
        format!("{}/code", self.remote.remote_base)
    }

    /// Chemin des jobs sur le serveur (partagé entre tous les projets)
    pub fn remote_jobs_path(&self) -> String {
        format!("{}/jobs", self.remote.remote_base)
    }

    /// Chemin des résultats sur le serveur
    pub fn remote_results_path(&self) -> String {
        format!("{}/results", self.remote.remote_base)
    }

    /// Chemin de la base de données (dans le dossier projet)
    pub fn db_path() -> Result<PathBuf, String> {
        Ok(Self::project_root().join("solver-pilot.db"))
    }
}
