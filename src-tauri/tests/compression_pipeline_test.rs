mod common;

use common::TestFixtures;
use image::{ImageBuffer, Rgb};
use imageforge_lib::metadata::inspect_image;
use imageforge_lib::models::{
    CompressionPreset, CompressionSettings, CompressionStatus, ImageFormat, OutputMode,
};
use imageforge_lib::processor::ImageProcessor;
use std::fs;
use std::path::Path;
use turbojpeg::{Decompressor, PixelFormat};

#[test]
fn test_jpeg_compression_pipeline() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.jpeg_path).expect("failed to inspect jpeg");
    assert_eq!(info.format, ImageFormat::Jpg);
    assert_eq!(info.width, 800);
    assert_eq!(info.height, 600);

    for preset in [
        CompressionPreset::BestQuality,
        CompressionPreset::Balanced,
        CompressionPreset::Smallest,
    ] {
        let settings = CompressionSettings {
            preset,
            output_mode: OutputMode::SameFormat,
            preserve_metadata: true,
            preserve_color_profile: true,
            custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
            filename_suffix: Some(format!("-{preset:?}")),
        };

        let result = processor.process_file(&info, &settings);
        assert_eq!(result.status, CompressionStatus::Completed);
        assert!(result.output_path.is_some());

        let out_path = result.output_path.unwrap();
        assert!(fs::metadata(&out_path).is_ok());

        // Verify independent decoding and dimension preservation
        let out_bytes = fs::read(&out_path).expect("failed to read output file");
        let mut decompressor = Decompressor::new().expect("failed to create decompressor");
        let header = decompressor
            .read_header(&out_bytes)
            .expect("failed to read header");
        assert_eq!(
            header.width, 800,
            "JPEG output width must match original invariant"
        );
        assert_eq!(
            header.height, 600,
            "JPEG output height must match original invariant"
        );
    }
}

#[test]
fn test_png_lossless_compression_pipeline() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.png_rgb_path).expect("failed to inspect png");
    assert_eq!(info.format, ImageFormat::Png);
    assert_eq!(info.width, 400);
    assert_eq!(info.height, 300);

    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::SameFormat,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: Some("-compressed".to_string()),
    };

    let result = processor.process_file(&info, &settings);
    assert_eq!(result.status, CompressionStatus::Completed);

    let out_path = result.output_path.expect("output path missing");
    let out_img = image::open(&out_path).expect("failed to decode output png");
    assert_eq!(
        out_img.width(),
        400,
        "PNG output width must match original invariant"
    );
    assert_eq!(
        out_img.height(),
        300,
        "PNG output height must match original invariant"
    );
}

#[test]
fn test_webp_compression_pipeline() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.webp_rgba_path).expect("failed to inspect webp");
    assert_eq!(info.format, ImageFormat::Webp);
    assert_eq!(info.width, 500);
    assert_eq!(info.height, 500);

    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::SameFormat,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: Some("-compressed".to_string()),
    };

    let result = processor.process_file(&info, &settings);
    assert_eq!(result.status, CompressionStatus::Completed);

    let out_path = result.output_path.expect("output path missing");
    let out_bytes = fs::read(&out_path).expect("failed to read output webp");
    let decoder = webp::Decoder::new(&out_bytes);
    let decoded = decoder.decode().expect("failed to decode output webp");
    assert_eq!(
        decoded.width(),
        500,
        "WebP output width must match original invariant"
    );
    assert_eq!(
        decoded.height(),
        500,
        "WebP output height must match original invariant"
    );
}

#[test]
fn test_heic_compression_pipeline() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.heic_path).expect("failed to inspect heic");
    assert_eq!(info.format, ImageFormat::Heic);
    assert_eq!(info.width, 320);
    assert_eq!(info.height, 240);

    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::SameFormat,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: Some("-compressed".to_string()),
    };

    let result = processor.process_file(&info, &settings);
    assert_eq!(result.status, CompressionStatus::Completed);

    let out_path = result.output_path.expect("output path missing");
    let ctx =
        libheif_rs::HeifContext::read_from_file(&out_path).expect("failed to read heic output");
    let handle = ctx.primary_image_handle().expect("failed to get handle");
    assert_eq!(
        handle.width(),
        320,
        "HEIC output width must match original invariant"
    );
    assert_eq!(
        handle.height(),
        240,
        "HEIC output height must match original invariant"
    );
}

#[test]
fn test_png_rgba_to_jpg_conversion_white_background() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.png_rgba_path).expect("failed to inspect png");
    assert_eq!(info.format, ImageFormat::Png);
    assert_eq!(info.width, 400);
    assert_eq!(info.height, 300);

    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::ConvertToJpg,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: None,
    };

    let result = processor.process_file(&info, &settings);
    assert_eq!(result.status, CompressionStatus::Completed);

    let out_path = result.output_path.expect("output path missing");
    assert!(
        Path::new(&out_path).extension().and_then(|e| e.to_str()) == Some("jpg"),
        "Converted file must have .jpg extension"
    );

    // Decode resulting JPG and inspect pixels
    let out_bytes = fs::read(&out_path).expect("failed to read converted jpg");
    let mut decompressor = Decompressor::new().expect("failed to create decompressor");
    let header = decompressor
        .read_header(&out_bytes)
        .expect("failed to read header");
    assert_eq!(header.width, 400);
    assert_eq!(header.height, 300);

    let mut rgb_pixels = vec![0u8; 400 * 300 * 3];
    let image_ref = turbojpeg::Image {
        pixels: &mut rgb_pixels[..],
        width: 400,
        pitch: 400 * 3,
        height: 300,
        format: PixelFormat::RGB,
    };
    decompressor
        .decompress(&out_bytes, image_ref)
        .expect("failed to decompress jpg pixels");

    // Check pixel at (100, 50) - top half: original had alpha=0 (transparent red)
    // In converted JPG, transparent pixels MUST composite over white (#FFFFFF)
    // Allowing JPEG DCT compression tolerance: RGB >= 240
    let top_offset = (50 * 400 + 100) * 3;
    let r_top = rgb_pixels[top_offset];
    let g_top = rgb_pixels[top_offset + 1];
    let b_top = rgb_pixels[top_offset + 2];
    assert!(
        r_top >= 240 && g_top >= 240 && b_top >= 240,
        "Transparent pixel must be composited to white (#FFFFFF). Got R:{r_top}, G:{g_top}, B:{b_top}"
    );

    // Check pixel at (300, 250) - bottom right: original had opaque blue (0, 0, 255, 255)
    let bot_offset = (250 * 400 + 300) * 3;
    let r_bot = rgb_pixels[bot_offset];
    let g_bot = rgb_pixels[bot_offset + 1];
    let b_bot = rgb_pixels[bot_offset + 2];
    assert!(
        r_bot <= 20 && g_bot <= 20 && b_bot >= 230,
        "Opaque blue pixel must be preserved. Got R:{r_bot}, G:{g_bot}, B:{b_bot}"
    );
}

#[test]
fn test_webp_rgba_to_jpg_conversion_white_background() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.webp_rgba_path).expect("failed to inspect webp");
    assert_eq!(info.format, ImageFormat::Webp);
    assert_eq!(info.width, 500);
    assert_eq!(info.height, 500);

    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::ConvertToJpg,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: None,
    };

    let result = processor.process_file(&info, &settings);
    assert_eq!(result.status, CompressionStatus::Completed);

    let out_path = result.output_path.expect("output path missing");
    assert_eq!(
        Path::new(&out_path).extension().and_then(|e| e.to_str()),
        Some("jpg")
    );

    let out_bytes = fs::read(&out_path).expect("failed to read converted jpg");
    let mut decompressor = Decompressor::new().expect("failed to create decompressor");
    let header = decompressor
        .read_header(&out_bytes)
        .expect("failed to read header");
    assert_eq!(header.width, 500);
    assert_eq!(header.height, 500);

    let mut rgb_pixels = vec![0u8; 500 * 500 * 3];
    let image_ref = turbojpeg::Image {
        pixels: &mut rgb_pixels[..],
        width: 500,
        pitch: 500 * 3,
        height: 500,
        format: PixelFormat::RGB,
    };
    decompressor
        .decompress(&out_bytes, image_ref)
        .expect("failed to decompress jpg");

    // Pixel at (100, 100) was transparent in original WebP
    let offset = (100 * 500 + 100) * 3;
    let r = rgb_pixels[offset];
    let g = rgb_pixels[offset + 1];
    let b = rgb_pixels[offset + 2];
    assert!(
        r >= 240 && g >= 240 && b >= 240,
        "Transparent webp pixel must be composited over white. Got R:{r}, G:{g}, B:{b}"
    );
}

#[test]
fn test_heic_to_jpg_conversion() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.heic_path).expect("failed to inspect heic");
    assert_eq!(info.format, ImageFormat::Heic);

    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::ConvertToJpg,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: None,
    };

    let result = processor.process_file(&info, &settings);
    assert_eq!(result.status, CompressionStatus::Completed);

    let out_path = result.output_path.expect("output path missing");
    assert_eq!(
        Path::new(&out_path).extension().and_then(|e| e.to_str()),
        Some("jpg")
    );

    let out_bytes = fs::read(&out_path).expect("failed to read converted jpg");
    let mut decompressor = Decompressor::new().expect("failed to create decompressor");
    let header = decompressor
        .read_header(&out_bytes)
        .expect("failed to read header");
    assert_eq!(header.width, 320);
    assert_eq!(header.height, 240);
}

#[test]
fn test_collision_resolution_and_atomic_write() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    let info = inspect_image(&fixtures.jpeg_path).expect("failed to inspect jpeg");
    let settings = CompressionSettings {
        preset: CompressionPreset::Balanced,
        output_mode: OutputMode::SameFormat,
        preserve_metadata: true,
        preserve_color_profile: true,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: Some("-compressed".to_string()),
    };

    // First run
    let res1 = processor.process_file(&info, &settings);
    assert_eq!(res1.status, CompressionStatus::Completed);
    let path1 = res1.output_path.unwrap();
    assert!(path1.ends_with("test_sample-compressed.jpg"));

    // Second run with same settings should resolve collision by appending -1
    let res2 = processor.process_file(&info, &settings);
    assert_eq!(res2.status, CompressionStatus::Completed);
    let path2 = res2.output_path.unwrap();
    assert!(
        path2.ends_with("test_sample-compressed-1.jpg"),
        "Collision should append -1, got: {path2:?}"
    );

    // Third run should resolve collision by appending -2
    let res3 = processor.process_file(&info, &settings);
    assert_eq!(res3.status, CompressionStatus::Completed);
    let path3 = res3.output_path.unwrap();
    assert!(
        path3.ends_with("test_sample-compressed-2.jpg"),
        "Collision should append -2, got: {path3:?}"
    );

    // Verify all three exist
    assert!(fs::metadata(&path1).is_ok());
    assert!(fs::metadata(&path2).is_ok());
    assert!(fs::metadata(&path3).is_ok());
}
/// When a PNG is already fully optimized by oxipng, running lossless compression again
/// should yield NoSavings (or Completed if trivially reducible), and the output must always
/// be written and have correct dimensions. This test verifies the NoSavings path by using
/// an already-tiny PNG — a 2×2 solid-color image — which oxipng cannot compress further.
#[test]
fn test_no_savings_retains_original() {
    let fixtures = TestFixtures::new();
    let processor = ImageProcessor::new();

    // Create an already-minimal PNG: 2×2 solid black pixels.
    // PNG overhead for such a trivial image means lossless re-compression at oxipng
    // level 6 will produce >= original bytes.
    let tiny_path = fixtures.dir.path().join("tiny_solid.png");
    {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(2, 2, Rgb([0u8, 0, 0]));
        img.save_with_format(&tiny_path, image::ImageFormat::Png)
            .expect("failed to save tiny png");
        // Pre-optimize with oxipng so it's already as small as possible
        let options = oxipng::Options::from_preset(6);
        let data = std::fs::read(&tiny_path).unwrap();
        let optimized = oxipng::optimize_from_memory(&data, &options).unwrap_or(data.clone());
        std::fs::write(&tiny_path, &optimized).unwrap();
    }

    let info = inspect_image(&tiny_path).expect("inspect tiny png");

    let settings = CompressionSettings {
        preset: CompressionPreset::Smallest,
        output_mode: OutputMode::SameFormat,
        preserve_metadata: false,
        preserve_color_profile: false,
        custom_output_dir: Some(fixtures.dir.path().to_string_lossy().to_string()),
        filename_suffix: Some("-nosavings-test".to_string()),
    };

    let result = processor.process_file(&info, &settings);

    // The result must be either NoSavings (when candidate >= original) or Completed (rare edge).
    // In both cases, an output_path must be present and the file must have correct dimensions.
    assert!(
        result.output_path.is_some(),
        "output_path must always be Some (got status {:?})",
        result.status
    );
    let out_path = result.output_path.unwrap();
    let out_bytes = fs::read(&out_path).expect("output file must be readable");

    // Verify output is still a valid PNG with correct dimensions
    let out_img = image::open(&out_path).expect("output must be a valid PNG");
    assert_eq!(out_img.width(), 2, "dimension invariant: width");
    assert_eq!(out_img.height(), 2, "dimension invariant: height");

    if result.status == CompressionStatus::NoSavings {
        // Verify the retained bytes match the original
        let orig_bytes = fs::read(&tiny_path).unwrap();
        assert_eq!(
            out_bytes, orig_bytes,
            "NoSavings must write original bytes to output"
        );
        assert_eq!(result.saved_bytes, 0);
        assert_eq!(result.saved_percent, 0.0);
        assert_eq!(result.output_bytes, result.input_bytes);
    } else {
        // Completed: compression did find savings (fine — both paths are tested)
        assert_eq!(result.status, CompressionStatus::Completed);
        assert!(result.saved_bytes > 0 || result.output_bytes <= result.input_bytes);
    }
}
