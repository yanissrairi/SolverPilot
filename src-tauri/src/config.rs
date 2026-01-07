use crate::paths;
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
    /// Charge la configuration depuis le répertoire de config système.
    ///
    /// Chemins par OS:
    /// - Linux: `~/.config/dev.yaniss.solver-pilot/config.toml`
    /// - macOS: `~/Library/Application Support/dev.yaniss.solver-pilot/config.toml`
    /// - Windows: `C:\Users\<user>\AppData\Roaming\dev.yaniss.solver-pilot\config.toml`
    ///
    /// Utilise zero-copy parsing (toml 0.9+) pour moins d'allocations mémoire.
    pub fn load() -> Result<Self, String> {
        let config_path = paths::config_path()?;

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

    /// Chemin de la base de données dans le répertoire de données système.
    ///
    /// Chemins par OS:
    /// - Linux: `~/.local/share/dev.yaniss.solver-pilot/solver-pilot.db`
    /// - macOS: `~/Library/Application Support/dev.yaniss.solver-pilot/solver-pilot.db`
    /// - Windows: `C:\Users\<user>\AppData\Roaming\dev.yaniss.solver-pilot\solver-pilot.db`
    pub fn db_path() -> Result<PathBuf, String> {
        paths::db_path()
    }
}
