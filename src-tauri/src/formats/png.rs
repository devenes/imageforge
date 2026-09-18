use oxipng::{Options, StripChunks};
use std::path::Path;

use super::ProcessedOutput;
use crate::errors::AppError;
use crate::models::{CompressionPreset, CompressionSettings, ImageFormat, ImageInfo};
use crate::validation::verify_dimensions;

pub fn compress_png(
    path: &Path,
    info: &ImageInfo,
    settings: &CompressionSettings,
) -> Result<ProcessedOutput, AppError> {
    let input_bytes = std::fs::read(path).map_err(|e| AppError::IoError(e.to_string()))?;

    // Configure OxiPNG options
    let mut options = match settings.preset {
        CompressionPreset::BestQuality => Options::from_preset(2),
        CompressionPreset::Balanced => Options::from_preset(4),
        CompressionPreset::Smallest => Options::from_preset(6),
    };

    options.interlace = None;

    // Headers & metadata
    if !settings.preserve_metadata {
        if settings.preserve_color_profile {
            let mut chunks = oxipng::IndexSet::new();
            chunks.insert(*b"iCCP");
            chunks.insert(*b"sRGB");
            chunks.insert(*b"cHRM");
            chunks.insert(*b"gAMA");
            options.strip = StripChunks::Keep(chunks);
        } else {
            options.strip = StripChunks::All;
        }
    } else {
        options.strip = StripChunks::None;
    }

    let optimized_data = oxipng::optimize_from_memory(&input_bytes, &options).map_err(|e| {
        AppError::encode("Failed to optimize PNG file.", format!("oxipng error: {e}"))
    })?;

    // Verify output dimensions without retaining borrow on optimized_data
    let (out_width, out_height) = {
        let cursor = std::io::Cursor::new(optimized_data.as_slice());
        let decoder = png::Decoder::new(cursor);
        let reader = decoder.read_info().map_err(|e| {
            AppError::decode(
                "Optimized PNG cannot be decoded by independent reader.",
                format!("png reader error: {e}"),
            )
        })?;
        let info_ref = reader.info();
        (info_ref.width, info_ref.height)
    };

    verify_dimensions(info.width, info.height, out_width, out_height)?;

    Ok(ProcessedOutput {
        data: optimized_data,
        width: out_width,
        height: out_height,
        format: ImageFormat::Png,
        color_profile_preserved: settings.preserve_color_profile,
    })
}
