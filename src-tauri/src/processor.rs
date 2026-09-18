use std::path::Path;
use std::time::Instant;
use tracing::{error, info};

use crate::formats::{CompressionStrategy, FixedQualityStrategy};
use crate::models::{
    CompressionResult, CompressionSettings, CompressionStatus, ImageInfo, OutputMode,
};
use crate::output::{atomic_write_file, resolve_output_path};
use crate::validation::calculate_savings;

pub struct ImageProcessor {
    strategy: Box<dyn CompressionStrategy>,
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageProcessor {
    pub fn new() -> Self {
        Self {
            strategy: Box::new(FixedQualityStrategy),
        }
    }

    pub fn process_file(
        &self,
        info: &ImageInfo,
        settings: &CompressionSettings,
    ) -> CompressionResult {
        let start_time = Instant::now();
        let input_path = Path::new(&info.path);

        if !input_path.exists() {
            return CompressionResult {
                id: info.id.clone(),
                input_bytes: info.bytes,
                output_bytes: 0,
                saved_bytes: 0,
                saved_percent: 0.0,
                input_width: info.width,
                input_height: info.height,
                output_width: 0,
                output_height: 0,
                input_format: info.format,
                output_format: match settings.output_mode {
                    OutputMode::SameFormat => info.format,
                    OutputMode::ConvertToJpg => crate::models::ImageFormat::Jpg,
                },
                output_path: None,
                duration_ms: 0,
                status: CompressionStatus::Failed,
                error: Some("Input file does not exist or has been moved.".to_string()),
                color_profile_preserved: false,
            };
        }

        info!(path = %info.path, format = ?info.format, "Beginning image compression");

        let processed = match self.strategy.process(input_path, info, settings) {
            Ok(p) => p,
            Err(err) => {
                error!(path = %info.path, error = %err, "Failed to compress image");
                return CompressionResult {
                    id: info.id.clone(),
                    input_bytes: info.bytes,
                    output_bytes: 0,
                    saved_bytes: 0,
                    saved_percent: 0.0,
                    input_width: info.width,
                    input_height: info.height,
                    output_width: 0,
                    output_height: 0,
                    input_format: info.format,
                    output_format: match settings.output_mode {
                        OutputMode::SameFormat => info.format,
                        OutputMode::ConvertToJpg => crate::models::ImageFormat::Jpg,
                    },
                    output_path: None,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    status: CompressionStatus::Failed,
                    error: Some(err.user_friendly_message()),
                    color_profile_preserved: false,
                };
            }
        };

        let output_bytes = processed.data.len() as u64;
        let (saved_bytes, saved_percent, keep_candidate) =
            calculate_savings(info.bytes, output_bytes, settings.output_mode);

        if !keep_candidate {
            // Compression didn't save bytes — write the original file bytes as output
            // so the user still receives a file in their chosen output folder.
            info!(path = %info.path, "No meaningful savings found; copying original to output");
            let original_data = match std::fs::read(input_path) {
                Ok(d) => d,
                Err(e) => {
                    return CompressionResult {
                        id: info.id.clone(),
                        input_bytes: info.bytes,
                        output_bytes: 0,
                        saved_bytes: 0,
                        saved_percent: 0.0,
                        input_width: info.width,
                        input_height: info.height,
                        output_width: 0,
                        output_height: 0,
                        input_format: info.format,
                        output_format: processed.format,
                        output_path: None,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        status: CompressionStatus::Failed,
                        error: Some(format!("Could not read original file: {e}")),
                        color_profile_preserved: false,
                    };
                }
            };
            let nosavings_output_path =
                match resolve_output_path(input_path, processed.format, settings) {
                    Ok(p) => p,
                    Err(e) => {
                        return CompressionResult {
                            id: info.id.clone(),
                            input_bytes: info.bytes,
                            output_bytes: 0,
                            saved_bytes: 0,
                            saved_percent: 0.0,
                            input_width: info.width,
                            input_height: info.height,
                            output_width: 0,
                            output_height: 0,
                            input_format: info.format,
                            output_format: processed.format,
                            output_path: None,
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            status: CompressionStatus::Failed,
                            error: Some(e.user_friendly_message()),
                            color_profile_preserved: false,
                        };
                    }
                };
            if let Err(e) = atomic_write_file(&nosavings_output_path, &original_data) {
                return CompressionResult {
                    id: info.id.clone(),
                    input_bytes: info.bytes,
                    output_bytes: 0,
                    saved_bytes: 0,
                    saved_percent: 0.0,
                    input_width: info.width,
                    input_height: info.height,
                    output_width: 0,
                    output_height: 0,
                    input_format: info.format,
                    output_format: processed.format,
                    output_path: None,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    status: CompressionStatus::Failed,
                    error: Some(e.user_friendly_message()),
                    color_profile_preserved: false,
                };
            }
            return CompressionResult {
                id: info.id.clone(),
                input_bytes: info.bytes,
                output_bytes: info.bytes,
                saved_bytes: 0,
                saved_percent: 0.0,
                input_width: info.width,
                input_height: info.height,
                output_width: processed.width,
                output_height: processed.height,
                input_format: info.format,
                output_format: processed.format,
                output_path: Some(nosavings_output_path.to_string_lossy().to_string()),
                duration_ms: start_time.elapsed().as_millis() as u64,
                status: CompressionStatus::NoSavings,
                error: None,
                color_profile_preserved: processed.color_profile_preserved,
            };
        }

        // Resolve output path & write atomically
        let output_path = match resolve_output_path(input_path, processed.format, settings) {
            Ok(p) => p,
            Err(e) => {
                return CompressionResult {
                    id: info.id.clone(),
                    input_bytes: info.bytes,
                    output_bytes,
                    saved_bytes,
                    saved_percent,
                    input_width: info.width,
                    input_height: info.height,
                    output_width: processed.width,
                    output_height: processed.height,
                    input_format: info.format,
                    output_format: processed.format,
                    output_path: None,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    status: CompressionStatus::Failed,
                    error: Some(e.user_friendly_message()),
                    color_profile_preserved: processed.color_profile_preserved,
                };
            }
        };

        if let Err(e) = atomic_write_file(&output_path, &processed.data) {
            return CompressionResult {
                id: info.id.clone(),
                input_bytes: info.bytes,
                output_bytes,
                saved_bytes,
                saved_percent,
                input_width: info.width,
                input_height: info.height,
                output_width: processed.width,
                output_height: processed.height,
                input_format: info.format,
                output_format: processed.format,
                output_path: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
                status: CompressionStatus::Failed,
                error: Some(e.user_friendly_message()),
                color_profile_preserved: processed.color_profile_preserved,
            };
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        info!(
            path = %info.path,
            output = %output_path.display(),
            saved_percent = %saved_percent,
            duration_ms = %duration_ms,
            "Successfully compressed image"
        );

        CompressionResult {
            id: info.id.clone(),
            input_bytes: info.bytes,
            output_bytes,
            saved_bytes,
            saved_percent,
            input_width: info.width,
            input_height: info.height,
            output_width: processed.width,
            output_height: processed.height,
            input_format: info.format,
            output_format: processed.format,
            output_path: Some(output_path.to_string_lossy().to_string()),
            duration_ms,
            status: CompressionStatus::Completed,
            error: None,
            color_profile_preserved: processed.color_profile_preserved,
        }
    }
}
