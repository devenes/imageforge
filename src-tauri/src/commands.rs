use std::path::Path;
use tauri::{AppHandle, State};
use tracing::{error, info};

use crate::metadata::inspect_image;
use crate::models::{CompressionSettings, ImageInfo};
use crate::queue::QueueManager;

#[tauri::command]
pub async fn inspect_files(paths: Vec<String>) -> Result<Vec<ImageInfo>, String> {
    let mut results = Vec::new();

    for path_str in paths {
        let path = Path::new(&path_str);
        if !path.exists() {
            continue;
        }

        match inspect_image(path) {
            Ok(info) => results.push(info),
            Err(err) => {
                error!(path = %path_str, error = %err, "Failed to inspect file");
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn start_batch(
    app: AppHandle,
    state: State<'_, QueueManager>,
    items: Vec<ImageInfo>,
    settings: CompressionSettings,
) -> Result<(), String> {
    info!(count = items.len(), preset = ?settings.preset, mode = ?settings.output_mode, "Starting compression batch");
    state.start_batch(app, items, settings, None).await;
    Ok(())
}

#[tauri::command]
pub async fn cancel_batch(state: State<'_, QueueManager>) -> Result<(), String> {
    info!("Cancelling compression batch");
    state.cancel_batch().await;
    Ok(())
}

#[tauri::command]
pub async fn open_output_folder(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let dir_to_open = if p.is_dir() {
        p
    } else if let Some(parent) = p.parent() {
        parent
    } else {
        Path::new(".")
    };

    open::that(dir_to_open).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reveal_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = open::that(&path);
        Ok(())
    }
}

#[tauri::command]
pub fn get_default_settings() -> CompressionSettings {
    CompressionSettings::default()
}
