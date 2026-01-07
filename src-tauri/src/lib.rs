pub mod commands;
pub mod config;
pub mod db;
pub mod job;
pub mod paths;
pub mod project;
pub mod python_deps;
pub mod ssh;
pub mod state;

use tauri::Manager;

/// Point d'entrée de l'application Tauri
///
/// # Errors
/// Retourne une erreur si l'initialisation de Tauri échoue
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize application paths (config, data directories)
            paths::init(app).map_err(|e| {
                tracing::error!("Failed to initialize paths: {e}");
                e
            })?;

            // Initialiser l'état de l'application
            app.manage(state::AppState::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            commands::load_config,
            // SSH
            commands::init_ssh,
            commands::close_ssh,
            commands::test_ssh,
            commands::check_ssh_key_status,
            commands::add_ssh_key,
            // Sync
            commands::check_sync_status,
            commands::sync_code,
            commands::sync_benchmark_deps,
            // Projects
            commands::list_projects,
            commands::create_project,
            commands::delete_project,
            commands::set_active_project,
            commands::get_active_project,
            // Python Versions
            commands::list_python_versions,
            commands::set_project_python_version,
            // Project Benchmarks
            commands::add_benchmark_to_project,
            commands::remove_benchmark_from_project,
            commands::list_project_benchmarks,
            commands::get_benchmark_dependencies,
            // Jobs
            commands::queue_jobs,
            commands::start_next_job,
            commands::stop_job,
            commands::kill_job,
            commands::get_job_logs,
            commands::get_job_status,
            // History
            commands::load_history,
            commands::delete_job,
            // Project Dependencies
            commands::add_project_dependency,
            commands::remove_project_dependency,
            commands::update_project_dependencies,
            commands::list_project_dependencies,
            commands::sync_project_environment,
        ])
        .run(tauri::generate_context!())
}
