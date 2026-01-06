use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub ssh: SshConfig,
    pub paths: PathsConfig,
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
    /// Chemin de la clé SSH (optionnel, défaut: ~/.`ssh/id_rsa`)
    #[serde(default = "default_key_path")]
    pub key_path: String,
}

fn default_key_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    format!("{home}/.ssh/id_rsa")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathsConfig {
    /// Dossier racine du code local (`3_ALGORITHMES`)
    pub local_code: PathBuf,
    /// Dossier des benchmarks locaux (pour affichage)
    pub local_benchmarks: PathBuf,
    /// Dossier des résultats locaux
    pub local_results: PathBuf,
    /// Dossier de base sur le serveur
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
    /// Charge la configuration depuis ~/.config/solver-pilot/config.toml
    pub fn load() -> Result<Self, String> {
        let config_dir = directories::ProjectDirs::from("", "", "solver-pilot")
            .ok_or("Impossible de déterminer le dossier config")?;

        let config_path = config_dir.config_dir().join("config.toml");

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Erreur lecture {}: {}", config_path.display(), e))?;

        toml::from_str(&content).map_err(|e| format!("Erreur parsing config.toml: {e}"))
    }

    /// Chemin du code sur le serveur
    pub fn remote_code_path(&self) -> String {
        format!("{}/code", self.paths.remote_base)
    }

    /// Chemin des jobs sur le serveur
    pub fn remote_jobs_path(&self) -> String {
        format!("{}/jobs", self.paths.remote_base)
    }

    /// Chemin des résultats sur le serveur
    pub fn remote_results_path(&self) -> String {
        format!("{}/results", self.paths.remote_base)
    }

    /// Chemin de la base de données
    pub fn db_path() -> Result<PathBuf, String> {
        let data_dir = directories::ProjectDirs::from("", "", "solver-pilot")
            .ok_or("Impossible de déterminer le dossier data")?;

        let db_dir = data_dir.data_dir();
        std::fs::create_dir_all(db_dir)
            .map_err(|e| format!("Erreur création dossier data: {e}"))?;

        Ok(db_dir.join("solver-pilot.db"))
    }
}
