use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::models::{CompressionResult, CompressionSettings, ImageInfo, ProgressUpdate};
use crate::processor::ImageProcessor;

pub struct QueueManager {
    current_token: Mutex<Option<CancellationToken>>,
    processor: Arc<ImageProcessor>,
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            current_token: Mutex::new(None),
            processor: Arc::new(ImageProcessor::new()),
        }
    }

    pub async fn cancel_batch(&self) {
        let mut token_guard = self.current_token.lock().await;
        if let Some(token) = token_guard.take() {
            info!("Cancelling current batch processing");
            token.cancel();
        }
    }

    pub async fn start_batch(
        &self,
        app_handle: AppHandle,
        items: Vec<ImageInfo>,
        settings: CompressionSettings,
        concurrency_override: Option<usize>,
    ) {
        // Cancel any existing batch
        self.cancel_batch().await;

        let token = CancellationToken::new();
        {
            let mut guard = self.current_token.lock().await;
            *guard = Some(token.clone());
        }

        let processor = Arc::clone(&self.processor);
        let concurrency = concurrency_override.unwrap_or_else(|| {
            let cpus = num_cpus::get();
            cpus.clamp(1, 6)
        });

        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(concurrency));
            let total_count = items.len();
            let mut join_set = tokio::task::JoinSet::new();

            for (index, item) in items.into_iter().enumerate() {
                if token.is_cancelled() {
                    break;
                }

                let sem_permit = match semaphore.clone().acquire_owned().await {
                    Ok(permit) => permit,
                    Err(_) => break,
                };

                let proc = Arc::clone(&processor);
                let app = app_handle.clone();
                let settings_clone = settings.clone();
                let cancel_token = token.clone();
                let filename = item.filename.clone();

                join_set.spawn(async move {
                    let _permit = sem_permit;

                    if cancel_token.is_cancelled() {
                        let res = CompressionResult {
                            id: item.id.clone(),
                            input_bytes: item.bytes,
                            output_bytes: 0,
                            saved_bytes: 0,
                            saved_percent: 0.0,
                            input_width: item.width,
                            input_height: item.height,
                            output_width: 0,
                            output_height: 0,
                            input_format: item.format,
                            output_format: item.format,
                            output_path: None,
                            duration_ms: 0,
                            status: crate::models::CompressionStatus::Skipped,
                            error: Some("Batch was cancelled.".to_string()),
                            color_profile_preserved: false,
                        };
                        let _ = app.emit("file_completed", &res);
                        return res;
                    }

                    // Emit start progress
                    let _ = app.emit(
                        "file_progress",
                        &ProgressUpdate {
                            current_index: index + 1,
                            total_count,
                            current_filename: filename,
                            stage: "Compressing".to_string(),
                        },
                    );

                    let result = tokio::task::spawn_blocking(move || {
                        proc.process_file(&item, &settings_clone)
                    })
                    .await
                    .unwrap_or_else(|e| CompressionResult {
                        id: "".to_string(),
                        input_bytes: 0,
                        output_bytes: 0,
                        saved_bytes: 0,
                        saved_percent: 0.0,
                        input_width: 0,
                        input_height: 0,
                        output_width: 0,
                        output_height: 0,
                        input_format: crate::models::ImageFormat::Jpg,
                        output_format: crate::models::ImageFormat::Jpg,
                        output_path: None,
                        duration_ms: 0,
                        status: crate::models::CompressionStatus::Failed,
                        error: Some(format!("Worker thread panicked: {e}")),
                        color_profile_preserved: false,
                    });

                    let _ = app.emit("file_completed", &result);
                    result
                });
            }

            let mut all_results = Vec::new();
            while let Some(res) = join_set.join_next().await {
                if let Ok(result) = res {
                    all_results.push(result);
                }
            }

            if token.is_cancelled() {
                let _ = app_handle.emit("batch_cancelled", ());
            } else {
                let _ = app_handle.emit("batch_completed", &all_results);
            }
        });
    }
}
