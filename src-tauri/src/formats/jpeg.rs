use bytes::Bytes;
use img_parts::{ImageEXIF, ImageICC};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use turbojpeg::{Compressor, Decompressor, PixelFormat, Subsamp};

use super::ProcessedOutput;
use crate::errors::AppError;
use crate::models::{CompressionPreset, CompressionSettings, ImageFormat, ImageInfo};
use crate::validation::verify_dimensions;

pub fn compress_jpeg(
    path: &Path,
    info: &ImageInfo,
    settings: &CompressionSettings,
) -> Result<ProcessedOutput, AppError> {
    let mut file = File::open(path).map_err(|e| AppError::IoError(e.to_string()))?;
    let mut input_data = Vec::new();
    file.read_to_end(&mut input_data)
        .map_err(|e| AppError::IoError(e.to_string()))?;

    // Decompress via libjpeg-turbo
    let mut decompressor = Decompressor::new().map_err(|e| {
        AppError::decode(
            "Could not initialize JPEG decompressor.",
            format!("turbojpeg Decompressor::new error: {e}"),
        )
    })?;

    let header = decompressor.read_header(&input_data).map_err(|e| {
        AppError::decode(
            "Could not read JPEG image header. File may be corrupted.",
            format!("turbojpeg read_header error: {e}"),
        )
    })?;

    let width = header.width;
    let height = header.height;

    // Verify input dimensions match inspected dimensions
    verify_dimensions(info.width, info.height, width as u32, height as u32)?;

    // Decompress into RGB buffer
    let mut rgb_pixels = vec![0u8; width * height * 3];
    let image_ref = turbojpeg::Image {
        pixels: rgb_pixels.as_mut_slice(),
        width,
        pitch: width * 3,
        height,
        format: PixelFormat::RGB,
    };

    decompressor
        .decompress(&input_data, image_ref)
        .map_err(|e| {
            AppError::decode(
                "Failed to decode JPEG image pixels.",
                format!("turbojpeg decompress error: {e}"),
            )
        })?;

    // Compress with libjpeg-turbo using presets
    let mut compressor = Compressor::new().map_err(|e| {
        AppError::encode(
            "Could not initialize JPEG compressor.",
            format!("turbojpeg Compressor::new error: {e}"),
        )
    })?;

    let (quality, subsample) = match settings.preset {
        CompressionPreset::BestQuality => (92, Subsamp::Sub2x2),
        CompressionPreset::Balanced => (84, Subsamp::Sub2x2),
        CompressionPreset::Smallest => (74, Subsamp::Sub2x2),
    };

    compressor.set_quality(quality).map_err(|e| {
        AppError::encode(
            "Could not set JPEG quality.",
            format!("turbojpeg set_quality error: {e}"),
        )
    })?;
    compressor.set_subsamp(subsample).map_err(|e| {
        AppError::encode(
            "Could not set JPEG chroma subsampling.",
            format!("turbojpeg set_subsamp error: {e}"),
        )
    })?;

    let rgb_input = turbojpeg::Image {
        pixels: rgb_pixels.as_slice(),
        width,
        pitch: width * 3,
        height,
        format: PixelFormat::RGB,
    };

    let encoded_buf = compressor.compress_to_vec(rgb_input).map_err(|e| {
        AppError::encode(
            "Failed to encode JPEG image.",
            format!("turbojpeg compress error: {e}"),
        )
    })?;

    let mut final_data = encoded_buf.to_vec();
    let mut color_profile_preserved = false;

    // Metadata & Color Profile handling
    if settings.preserve_metadata || settings.preserve_color_profile {
        let in_bytes = Bytes::from(input_data);
        if let Ok(orig_jpeg) = img_parts::jpeg::Jpeg::from_bytes(in_bytes) {
            let out_bytes = Bytes::from(final_data.clone());
            if let Ok(mut new_jpeg) = img_parts::jpeg::Jpeg::from_bytes(out_bytes) {
                // Transfer ICC profile
                if settings.preserve_color_profile {
                    if let Some(icc) = orig_jpeg.icc_profile() {
                        new_jpeg.set_icc_profile(Some(icc));
                        color_profile_preserved = true;
                    }
                }
                // Transfer EXIF
                if settings.preserve_metadata {
                    if let Some(exif) = orig_jpeg.exif() {
                        new_jpeg.set_exif(Some(exif));
                    }
                }
                let mut out_vec = Vec::new();
                if new_jpeg.encoder().write_to(&mut out_vec).is_ok() {
                    final_data = out_vec;
                }
            }
        }
    }

    // Double check output dimensions
    let mut verify_dec = Decompressor::new().map_err(|e| {
        AppError::decode(
            "Failed to verify output JPEG.",
            format!("turbojpeg verify decompressor: {e}"),
        )
    })?;
    let out_header = verify_dec.read_header(&final_data).map_err(|e| {
        AppError::decode(
            "Compressed JPEG cannot be decoded by independent reader.",
            format!("turbojpeg verify read_header: {e}"),
        )
    })?;

    verify_dimensions(
        info.width,
        info.height,
        out_header.width as u32,
        out_header.height as u32,
    )?;

    Ok(ProcessedOutput {
        data: final_data,
        width: out_header.width as u32,
        height: out_header.height as u32,
        format: ImageFormat::Jpg,
        color_profile_preserved,
    })
}
