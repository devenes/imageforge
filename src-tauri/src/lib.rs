pub mod commands;
pub mod errors;
pub mod formats;
pub mod metadata;
pub mod models;
pub mod output;
pub mod processor;
pub mod queue;
pub mod validation;

use queue::QueueManager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Structured logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,imageforge=debug"));
    let _ = tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .try_init();

    let queue_manager = QueueManager::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(queue_manager)
        .invoke_handler(tauri::generate_handler![
            commands::inspect_files,
            commands::start_batch,
            commands::cancel_batch,
            commands::open_output_folder,
            commands::reveal_file,
            commands::get_default_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ImageForge application");
}
