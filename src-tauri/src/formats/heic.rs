use libheif_rs::{ColorSpace, CompressionFormat, HeifContext, LibHeif, RgbChroma};
use std::path::Path;

use super::ProcessedOutput;
use crate::errors::AppError;
use crate::models::{CompressionPreset, CompressionSettings, ImageFormat, ImageInfo};
use crate::validation::verify_dimensions;

pub fn compress_heic(
    path: &Path,
    info: &ImageInfo,
    settings: &CompressionSettings,
) -> Result<ProcessedOutput, AppError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| AppError::IoError("Invalid non-UTF8 path for HEIC file".into()))?;

    let in_ctx = HeifContext::read_from_file(path_str).map_err(|e| {
        AppError::decode(
            "Could not read this HEIC file. The file may be damaged or use an unsupported codec.",
            format!("libheif read_from_file error: {e}"),
        )
    })?;

    let handle = in_ctx.primary_image_handle().map_err(|e| {
        AppError::decode(
            "Could not locate primary image in HEIC container.",
            format!("libheif primary_image_handle error: {e}"),
        )
    })?;

    let width = handle.width();
    let height = handle.height();
    let has_alpha = handle.has_alpha_channel();

    verify_dimensions(info.width, info.height, width, height)?;

    let lib_heif = LibHeif::new();

    // Decode to RGB or RGBA image
    let chroma = if has_alpha {
        RgbChroma::Rgba
    } else {
        RgbChroma::Rgb
    };

    let decoded_image = lib_heif
        .decode(&handle, ColorSpace::Rgb(chroma), None)
        .map_err(|e| {
            AppError::decode(
                "Failed to decode HEIC pixels.",
                format!("libheif decode error: {e}"),
            )
        })?;

    // Prepare output context and encoder
    let mut out_ctx = HeifContext::new().map_err(|e| {
        AppError::encode(
            "Failed to initialize HEIF context for encoding.",
            format!("libheif HeifContext::new error: {e}"),
        )
    })?;

    let mut encoder = lib_heif
        .encoder_for_format(CompressionFormat::Hevc)
        .map_err(|e| {
            AppError::encode(
                "HEVC encoder is not available in the current libheif setup.",
                format!("libheif encoder_for_format(Hevc) error: {e}"),
            )
        })?;

    let quality = match settings.preset {
        CompressionPreset::BestQuality => 88,
        CompressionPreset::Balanced => 78,
        CompressionPreset::Smallest => 65,
    };

    let _ = encoder.set_quality(libheif_rs::EncoderQuality::Lossy(quality));

    // Encode image
    let mut encoding_options = libheif_rs::EncodingOptions::new()
        .map_err(|e| AppError::encode("Failed to create HEIC encoding options", e.to_string()))?;
    encoding_options.set_save_alpha_channel(has_alpha);
    encoding_options.set_mac_os_compatibility_workaround(true);

    let out_handle = out_ctx
        .encode_image(&decoded_image, &mut encoder, Some(encoding_options))
        .map_err(|e| {
            AppError::encode(
                "Failed to encode HEIC image.",
                format!("libheif encode_image error: {e}"),
            )
        })?;

    // Transfer EXIF metadata if enabled
    if settings.preserve_metadata {
        let num_blocks = handle.number_of_metadata_blocks(b"Exif");
        if num_blocks > 0 {
            let mut ids = vec![0; num_blocks as usize];
            let real_count = handle.metadata_block_ids(&mut ids, b"Exif");
            if real_count > 0 {
                if let Ok(exif_bytes) = handle.metadata(ids[0]) {
                    let _ = out_ctx.add_exif_metadata(&out_handle, &exif_bytes);
                }
            }
        }
    }

    let encoded_data = out_ctx.write_to_bytes().map_err(|e| {
        AppError::encode(
            "Failed to write HEIC output data.",
            format!("libheif write_to_bytes error: {e}"),
        )
    })?;

    // Verify independently within a scoped block to release the borrow before returning
    let (verify_w, verify_h) = {
        let mut verify_ctx = HeifContext::new()
            .map_err(|e| AppError::decode("Failed to create verify context", e.to_string()))?;
        verify_ctx.read_bytes(&encoded_data).map_err(|e| {
            AppError::decode(
                "Encoded HEIC cannot be verified by independent reader.",
                format!("libheif verify read_bytes error: {e}"),
            )
        })?;
        let verify_handle = verify_ctx.primary_image_handle().map_err(|e| {
            AppError::decode(
                "Encoded HEIC missing primary image.",
                format!("libheif verify primary_image_handle error: {e}"),
            )
        })?;
        (verify_handle.width(), verify_handle.height())
    };

    verify_dimensions(info.width, info.height, verify_w, verify_h)?;

    Ok(ProcessedOutput {
        data: encoded_data,
        width: verify_w,
        height: verify_h,
        format: ImageFormat::Heic,
        color_profile_preserved: settings.preserve_color_profile,
    })
}
