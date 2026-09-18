use bytes::Bytes;
use img_parts::{ImageEXIF, ImageICC};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{ImageFormat, ImageInfo};

/// Sniff image format from file header magic bytes or fallback to extension
pub fn detect_format(path: &Path) -> Result<ImageFormat, AppError> {
    let mut file = File::open(path).map_err(|e| AppError::IoError(e.to_string()))?;
    let mut header = [0u8; 32];
    let bytes_read = file.read(&mut header).unwrap_or(0);

    if bytes_read >= 3 && header[0..3] == [0xFF, 0xD8, 0xFF] {
        return Ok(ImageFormat::Jpg);
    }

    if bytes_read >= 8 && header[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Ok(ImageFormat::Png);
    }

    if bytes_read >= 12 && &header[0..4] == b"RIFF" && &header[8..12] == b"WEBP" {
        return Ok(ImageFormat::Webp);
    }

    // HEIF / HEIC: Box format starts with 4 bytes size, then "ftyp", then brand
    if bytes_read >= 12 && &header[4..8] == b"ftyp" {
        let brand = &header[8..12];
        if brand == b"heic"
            || brand == b"heix"
            || brand == b"hevc"
            || brand == b"heim"
            || brand == b"heis"
            || brand == b"mif1"
            || brand == b"msf1"
        {
            return Ok(ImageFormat::Heic);
        }
    }

    // Fallback to extension check
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => return Ok(ImageFormat::Jpg),
            "png" => return Ok(ImageFormat::Png),
            "webp" => return Ok(ImageFormat::Webp),
            "heic" | "heif" => return Ok(ImageFormat::Heic),
            _ => {}
        }
    }

    Err(AppError::UnsupportedFormat(
        path.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown")
            .to_string(),
    ))
}

/// Inspect image file and extract metadata, dimensions, and color profiles
pub fn inspect_image(path: &Path) -> Result<ImageInfo, AppError> {
    let format = detect_format(path)?;
    let metadata = std::fs::metadata(path).map_err(|e| AppError::IoError(e.to_string()))?;
    let file_size = metadata.len();
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    let mut orientation = None;
    let mut color_profile = None;
    let mut has_alpha = false;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut metadata_available = false;

    // Read EXIF if present
    if let Ok(file) = File::open(path) {
        let mut buf_reader = BufReader::new(file);
        let exif_reader = exif::Reader::new();
        if let Ok(exif_data) = exif_reader.read_from_container(&mut buf_reader) {
            metadata_available = true;
            if let Some(field) = exif_data.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
                if let Some(val) = field.value.get_uint(0) {
                    orientation = Some(val);
                }
            }
        }
    }

    match format {
        ImageFormat::Jpg => {
            let file = File::open(path).map_err(|e| AppError::IoError(e.to_string()))?;
            let mut reader = BufReader::new(file);
            let mut data = Vec::with_capacity(std::cmp::min(file_size as usize, 1024 * 1024 * 32));
            reader
                .read_to_end(&mut data)
                .map_err(|e| AppError::IoError(e.to_string()))?;

            if let Ok(mut decompressor) = turbojpeg::Decompressor::new() {
                if let Ok(header) = decompressor.read_header(&data) {
                    width = header.width as u32;
                    height = header.height as u32;
                }
            }
            if width == 0 || height == 0 {
                if let Ok(dim) = image::image_dimensions(path) {
                    width = dim.0;
                    height = dim.1;
                }
            }

            // Check ICC profile via img-parts
            let b = Bytes::from(data);
            if let Ok(jpeg) = img_parts::jpeg::Jpeg::from_bytes(b) {
                if jpeg.icc_profile().is_some() {
                    color_profile = Some("Embedded ICC".to_string());
                    metadata_available = true;
                }
            }
            has_alpha = false;
        }
        ImageFormat::Png => {
            if let Ok(dim) = image::image_dimensions(path) {
                width = dim.0;
                height = dim.1;
            }
            // Check alpha channel and metadata using png decoder
            if let Ok(file) = File::open(path) {
                let decoder = png::Decoder::new(BufReader::new(file));
                if let Ok(reader) = decoder.read_info() {
                    let info = reader.info();
                    has_alpha = matches!(
                        info.color_type,
                        png::ColorType::Rgba | png::ColorType::GrayscaleAlpha
                    );
                    if info.icc_profile.is_some() {
                        color_profile = Some("Embedded ICC".to_string());
                        metadata_available = true;
                    } else if info.srgb.is_some() {
                        color_profile = Some("sRGB".to_string());
                    }
                }
            }
        }
        ImageFormat::Webp => {
            let data = std::fs::read(path).map_err(|e| AppError::IoError(e.to_string()))?;
            let decoder = webp::Decoder::new(&data);
            if let Some(image) = decoder.decode() {
                width = image.width();
                height = image.height();
                has_alpha = image.is_alpha();
            } else if let Ok(dim) = image::image_dimensions(path) {
                width = dim.0;
                height = dim.1;
            }

            let b = Bytes::from(data);
            if let Ok(w) = img_parts::webp::WebP::from_bytes(b) {
                if w.icc_profile().is_some() {
                    color_profile = Some("Embedded ICC".to_string());
                    metadata_available = true;
                }
                if w.exif().is_some() {
                    metadata_available = true;
                }
            }
        }
        ImageFormat::Heic => {
            match libheif_rs::HeifContext::read_from_file(path.to_str().unwrap_or_default()) {
                Ok(context) => {
                    if let Ok(handle) = context.primary_image_handle() {
                        width = handle.width();
                        height = handle.height();
                        has_alpha = handle.has_alpha_channel();
                        let num_blocks = handle.number_of_metadata_blocks(0u32);
                        if num_blocks > 0 {
                            metadata_available = true;
                        }
                        if handle.color_profile_raw().is_some() {
                            color_profile = Some("Embedded Color Profile".to_string());
                        }
                    }
                }
                Err(e) => {
                    return Err(AppError::decode(
                        "Could not read this HEIC file. The file may be damaged or use an unsupported codec.",
                        format!("libheif context error: {e}"),
                    ));
                }
            }
        }
    }

    if width == 0 || height == 0 {
        return Err(AppError::decode(
            format!("Could not determine dimensions for {filename}."),
            "Image dimensions are 0x0 or corrupted",
        ));
    }

    Ok(ImageInfo {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        filename,
        format,
        width,
        height,
        bytes: file_size,
        orientation,
        has_alpha,
        color_profile,
        metadata_available,
    })
}
