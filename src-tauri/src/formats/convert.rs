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

/// Alpha composites an RGBA buffer over an opaque solid white (#FFFFFF) background.
/// Formula: output_channel = (alpha * color + (255 - alpha) * 255 + 127) / 255
pub fn composite_rgba_onto_white(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(width * height * 3);
    for chunk in rgba.as_chunks::<4>().0 {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        let a = chunk[3] as u32;

        if a == 255 {
            rgb.push(r as u8);
            rgb.push(g as u8);
            rgb.push(b as u8);
        } else if a == 0 {
            // Fully transparent -> solid white
            rgb.push(255);
            rgb.push(255);
            rgb.push(255);
        } else {
            // Premultiplied alpha blending over white (255, 255, 255)
            let out_r = ((r * a + 255 * (255 - a) + 127) / 255) as u8;
            let out_g = ((g * a + 255 * (255 - a) + 127) / 255) as u8;
            let out_b = ((b * a + 255 * (255 - a) + 127) / 255) as u8;
            rgb.push(out_r);
            rgb.push(out_g);
            rgb.push(out_b);
        }
    }
    rgb
}

/// Convert any supported format (JPG, PNG, WebP, HEIC) to JPG.
pub fn convert_to_jpg(
    path: &Path,
    info: &ImageInfo,
    settings: &CompressionSettings,
) -> Result<ProcessedOutput, AppError> {
    let (width, height, rgb_pixels, icc_data, exif_data) = match info.format {
        ImageFormat::Jpg => {
            let mut file = File::open(path).map_err(|e| AppError::IoError(e.to_string()))?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| AppError::IoError(e.to_string()))?;

            let mut decompressor = Decompressor::new()
                .map_err(|e| AppError::decode("Failed to init JPEG decompressor", e.to_string()))?;
            let header = decompressor
                .read_header(&data)
                .map_err(|e| AppError::decode("Failed to read JPEG header", e.to_string()))?;
            let w = header.width;
            let h = header.height;
            let mut rgb = vec![0u8; w * h * 3];
            decompressor
                .decompress(
                    &data,
                    turbojpeg::Image {
                        pixels: rgb.as_mut_slice(),
                        width: w,
                        pitch: w * 3,
                        height: h,
                        format: PixelFormat::RGB,
                    },
                )
                .map_err(|e| AppError::decode("Failed to decompress JPEG", e.to_string()))?;

            let mut icc = None;
            let mut exif = None;
            let b = Bytes::from(data);
            if let Ok(jpeg) = img_parts::jpeg::Jpeg::from_bytes(b) {
                icc = jpeg.icc_profile().map(|b| b.to_vec());
                exif = jpeg.exif().map(|b| b.to_vec());
            }

            (w as u32, h as u32, rgb, icc, exif)
        }
        ImageFormat::Png => {
            let img = image::open(path)
                .map_err(|e| AppError::decode("Failed to read PNG image file", e.to_string()))?;
            let w = img.width();
            let h = img.height();

            let rgba = img.to_rgba8();
            let rgb = composite_rgba_onto_white(rgba.as_raw(), w as usize, h as usize);
            (w, h, rgb, None, None)
        }
        ImageFormat::Webp => {
            let data = std::fs::read(path).map_err(|e| AppError::IoError(e.to_string()))?;
            let decoder = webp::Decoder::new(&data);
            let image = decoder.decode().ok_or_else(|| {
                AppError::decode("Could not decode WebP image", "libwebp decode failed")
            })?;

            let w = image.width();
            let h = image.height();

            let rgb = if image.is_alpha() {
                composite_rgba_onto_white(&image, w as usize, h as usize)
            } else {
                image.to_vec()
            };

            let mut icc = None;
            let mut exif = None;
            let b = Bytes::from(data);
            if let Ok(w_part) = img_parts::webp::WebP::from_bytes(b) {
                icc = w_part.icc_profile().map(|p| p.to_vec());
                exif = w_part.exif().map(|e| e.to_vec());
            }

            (w, h, rgb, icc, exif)
        }
        ImageFormat::Heic => {
            let path_str = path
                .to_str()
                .ok_or_else(|| AppError::IoError("Invalid non-UTF8 path for HEIC".into()))?;
            let in_ctx = libheif_rs::HeifContext::read_from_file(path_str)
                .map_err(|e| AppError::decode("Could not read HEIC file", e.to_string()))?;
            let handle = in_ctx.primary_image_handle().map_err(|e| {
                AppError::decode("Could not locate primary HEIC image", e.to_string())
            })?;

            let w = handle.width();
            let h = handle.height();
            let has_alpha = handle.has_alpha_channel();
            let lib_heif = libheif_rs::LibHeif::new();

            let rgb = if has_alpha {
                let decoded = lib_heif
                    .decode(
                        &handle,
                        libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::Rgba),
                        None,
                    )
                    .map_err(|e| AppError::decode("Failed to decode HEIC RGBA", e.to_string()))?;

                let planes = decoded.planes();
                let plane = planes
                    .interleaved
                    .ok_or_else(|| AppError::decode("Failed to get HEIC interleaved plane", ""))?;

                let mut rgba_unpadded = Vec::with_capacity((w * h * 4) as usize);
                for y in 0..h {
                    let start = (y as usize) * plane.stride;
                    let end = start + (w as usize) * 4;
                    rgba_unpadded.extend_from_slice(&plane.data[start..end]);
                }

                composite_rgba_onto_white(&rgba_unpadded, w as usize, h as usize)
            } else {
                let decoded = lib_heif
                    .decode(
                        &handle,
                        libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::Rgb),
                        None,
                    )
                    .map_err(|e| AppError::decode("Failed to decode HEIC RGB", e.to_string()))?;

                let planes = decoded.planes();
                let plane = planes
                    .interleaved
                    .ok_or_else(|| AppError::decode("Failed to get HEIC interleaved plane", ""))?;

                let mut rgb_unpadded = Vec::with_capacity((w * h * 3) as usize);
                for y in 0..h {
                    let start = (y as usize) * plane.stride;
                    let end = start + (w as usize) * 3;
                    rgb_unpadded.extend_from_slice(&plane.data[start..end]);
                }

                rgb_unpadded
            };

            let mut exif = None;
            let num_blocks = handle.number_of_metadata_blocks(b"Exif");
            if num_blocks > 0 {
                let mut ids = vec![0; num_blocks as usize];
                let real_count = handle.metadata_block_ids(&mut ids, b"Exif");
                if real_count > 0 {
                    if let Ok(exif_bytes) = handle.metadata(ids[0]) {
                        exif = Some(exif_bytes);
                    }
                }
            }

            (w, h, rgb, None, exif)
        }
    };

    verify_dimensions(info.width, info.height, width, height)?;

    // Encode into JPEG using libjpeg-turbo
    let mut compressor = Compressor::new()
        .map_err(|e| AppError::encode("Failed to init JPEG compressor", e.to_string()))?;

    let (quality, subsample) = match settings.preset {
        CompressionPreset::BestQuality => (92, Subsamp::Sub2x2),
        CompressionPreset::Balanced => (84, Subsamp::Sub2x2),
        CompressionPreset::Smallest => (74, Subsamp::Sub2x2),
    };

    compressor
        .set_quality(quality)
        .map_err(|e| AppError::encode("Failed to set JPEG quality", e.to_string()))?;
    compressor
        .set_subsamp(subsample)
        .map_err(|e| AppError::encode("Failed to set JPEG subsampling", e.to_string()))?;

    let rgb_input = turbojpeg::Image {
        pixels: rgb_pixels.as_slice(),
        width: width as usize,
        pitch: width as usize * 3,
        height: height as usize,
        format: PixelFormat::RGB,
    };

    let encoded_buf = compressor
        .compress_to_vec(rgb_input)
        .map_err(|e| AppError::encode("Failed to encode JPEG during conversion", e.to_string()))?;

    let mut final_data = encoded_buf.to_vec();
    let mut color_profile_preserved = false;

    // Inject ICC / EXIF if enabled
    if (settings.preserve_color_profile && icc_data.is_some())
        || (settings.preserve_metadata && exif_data.is_some())
    {
        let b = Bytes::from(final_data.clone());
        if let Ok(mut new_jpeg) = img_parts::jpeg::Jpeg::from_bytes(b) {
            if settings.preserve_color_profile {
                if let Some(icc) = icc_data {
                    new_jpeg.set_icc_profile(Some(Bytes::from(icc)));
                    color_profile_preserved = true;
                }
            }
            if settings.preserve_metadata {
                if let Some(exif) = exif_data {
                    new_jpeg.set_exif(Some(Bytes::from(exif)));
                }
            }
            let mut out = Vec::new();
            if new_jpeg.encoder().write_to(&mut out).is_ok() {
                final_data = out;
            }
        }
    }

    // Verify output independently
    let mut verify_dec = Decompressor::new()
        .map_err(|e| AppError::decode("Failed to verify converted JPEG", e.to_string()))?;
    let out_header = verify_dec
        .read_header(&final_data)
        .map_err(|e| AppError::decode("Converted JPEG is invalid", e.to_string()))?;

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
