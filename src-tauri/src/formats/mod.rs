pub mod convert;
pub mod heic;
pub mod jpeg;
pub mod png;
pub mod webp;

use crate::errors::AppError;
use crate::models::{CompressionSettings, ImageFormat, ImageInfo, OutputMode};
use std::path::Path;

pub struct ProcessedOutput {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub color_profile_preserved: bool,
}

/// Compression strategy abstraction allowing for preset-based and future smart-perceptual strategies.
pub trait CompressionStrategy: Send + Sync {
    fn process(
        &self,
        path: &Path,
        info: &ImageInfo,
        settings: &CompressionSettings,
    ) -> Result<ProcessedOutput, AppError>;
}

/// Production strategy based on tuned format presets (BestQuality, Balanced, Smallest)
pub struct FixedQualityStrategy;

impl CompressionStrategy for FixedQualityStrategy {
    fn process(
        &self,
        path: &Path,
        info: &ImageInfo,
        settings: &CompressionSettings,
    ) -> Result<ProcessedOutput, AppError> {
        match settings.output_mode {
            OutputMode::ConvertToJpg => convert::convert_to_jpg(path, info, settings),
            OutputMode::SameFormat => match info.format {
                ImageFormat::Jpg => jpeg::compress_jpeg(path, info, settings),
                ImageFormat::Png => png::compress_png(path, info, settings),
                ImageFormat::Webp => webp::compress_webp(path, info, settings),
                ImageFormat::Heic => heic::compress_heic(path, info, settings),
            },
        }
    }
}
