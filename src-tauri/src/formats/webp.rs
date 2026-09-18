use bytes::Bytes;
use img_parts::{ImageEXIF, ImageICC};
use std::path::Path;
use webp::{Decoder, Encoder};

use super::ProcessedOutput;
use crate::errors::AppError;
use crate::models::{CompressionPreset, CompressionSettings, ImageFormat, ImageInfo};
use crate::validation::verify_dimensions;

pub fn compress_webp(
    path: &Path,
    info: &ImageInfo,
    settings: &CompressionSettings,
) -> Result<ProcessedOutput, AppError> {
    let input_bytes = std::fs::read(path).map_err(|e| AppError::IoError(e.to_string()))?;

    // Decode with libwebp
    let decoder = Decoder::new(&input_bytes);
    let image = decoder.decode().ok_or_else(|| {
        AppError::decode(
            "Could not decode WebP image. File may be corrupted or invalid.",
            "libwebp Decoder::decode returned None",
        )
    })?;

    let width = image.width();
    let height = image.height();

    verify_dimensions(info.width, info.height, width, height)?;

    // Encode with libwebp
    let encoder = if image.is_alpha() {
        Encoder::from_rgba(&image, width, height)
    } else {
        Encoder::from_rgb(&image, width, height)
    };

    // WebP preset mapping
    let webp_memory = match settings.preset {
        CompressionPreset::BestQuality => encoder.encode(92.0),
        CompressionPreset::Balanced => encoder.encode(82.0),
        CompressionPreset::Smallest => encoder.encode(70.0),
    };

    let mut final_data = webp_memory.to_vec();
    let mut color_profile_preserved = false;

    // Preserve metadata & color profile if present
    if settings.preserve_metadata || settings.preserve_color_profile {
        let orig_b = Bytes::from(input_bytes);
        if let Ok(orig_webp) = img_parts::webp::WebP::from_bytes(orig_b) {
            let new_b = Bytes::from(final_data.clone());
            if let Ok(mut new_webp) = img_parts::webp::WebP::from_bytes(new_b) {
                if settings.preserve_color_profile {
                    if let Some(icc) = orig_webp.icc_profile() {
                        new_webp.set_icc_profile(Some(icc));
                        color_profile_preserved = true;
                    }
                }
                if settings.preserve_metadata {
                    if let Some(exif) = orig_webp.exif() {
                        new_webp.set_exif(Some(exif));
                    }
                }
                let mut out_buf = Vec::new();
                if new_webp.encoder().write_to(&mut out_buf).is_ok() {
                    final_data = out_buf;
                }
            }
        }
    }

    // Verify output dimensions independently
    let verify_dec = Decoder::new(&final_data);
    let verify_img = verify_dec.decode().ok_or_else(|| {
        AppError::decode(
            "Compressed WebP cannot be decoded by independent reader.",
            "Decoder::decode returned None on verified output",
        )
    })?;

    verify_dimensions(
        info.width,
        info.height,
        verify_img.width(),
        verify_img.height(),
    )?;

    Ok(ProcessedOutput {
        data: final_data,
        width: verify_img.width(),
        height: verify_img.height(),
        format: ImageFormat::Webp,
        color_profile_preserved,
    })
}
