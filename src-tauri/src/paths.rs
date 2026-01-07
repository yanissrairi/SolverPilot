//! Application paths management using Tauri's `PathResolver`.
//!
//! This module provides centralized access to application directories
//! following platform conventions (XDG on Linux, Application Support on macOS, etc.)

use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::{App, Manager};

/// Application paths initialized at startup
static PATHS: OnceLock<AppPaths> = OnceLock::new();

/// Holds resolved application paths
#[derive(Debug, Clone)]
pub struct AppPaths {
    /// Directory for configuration files (config.toml)
    pub config_dir: PathBuf,
    /// Directory for data files (solver-pilot.db)
    pub data_dir: PathBuf,
}

/// Initialize application paths from Tauri's `PathResolver`.
///
/// Must be called once during app setup. Creates directories if they don't exist.
///
/// # Errors
/// Returns an error if paths cannot be resolved or directories cannot be created.
pub fn init(app: &App) -> Result<(), String> {
    let path_resolver = app.path();

    let config_dir = path_resolver
        .app_config_dir()
        .map_err(|e| format!("Failed to resolve config directory: {e}"))?;

    let data_dir = path_resolver
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve data directory: {e}"))?;

    // Create directories if they don't exist
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {e}"))?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {e}"))?;

    let paths = AppPaths {
        config_dir: config_dir.clone(),
        data_dir: data_dir.clone(),
    };

    tracing::info!("App paths initialized:");
    tracing::info!("  Config: {}", config_dir.display());
    tracing::info!("  Data:   {}", data_dir.display());

    PATHS
        .set(paths)
        .map_err(|_| "Paths already initialized".to_string())?;

    Ok(())
}

/// Get the path to the configuration file.
///
/// # Errors
/// Returns an error if `init()` was not called.
pub fn config_path() -> Result<PathBuf, String> {
    Ok(PATHS
        .get()
        .ok_or("paths::init() must be called before config_path()")?
        .config_dir
        .join("config.toml"))
}

/// Get the path to the database file.
///
/// # Errors
/// Returns an error if `init()` was not called.
pub fn db_path() -> Result<PathBuf, String> {
    Ok(PATHS
        .get()
        .ok_or("paths::init() must be called before db_path()")?
        .data_dir
        .join("solver-pilot.db"))
}

/// Get the path to the projects directory.
///
/// # Errors
/// Returns an error if `init()` was not called.
pub fn projects_dir() -> Result<PathBuf, String> {
    Ok(PATHS
        .get()
        .ok_or("paths::init() must be called before projects_dir()")?
        .data_dir
        .join("projects"))
}

/// Get the application paths.
///
/// # Errors
/// Returns an error if `init()` was not called.
pub fn get() -> Result<&'static AppPaths, String> {
    PATHS
        .get()
        .ok_or_else(|| "paths::init() must be called before get()".to_string())
}
