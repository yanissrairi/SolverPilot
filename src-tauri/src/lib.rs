pub mod commands;
pub mod config;
pub mod db;
pub mod job;
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
        .setup(|app| {
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
            // Benchmarks
            commands::scan_benchmarks,
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
        ])
        .run(tauri::generate_context!())
}
