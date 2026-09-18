use image::{ImageBuffer, Rgb, Rgba};
use libheif_rs::{Channel, ColorSpace, CompressionFormat, HeifContext, LibHeif, RgbChroma};
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestFixtures {
    pub dir: TempDir,
    pub jpeg_path: PathBuf,
    pub png_rgb_path: PathBuf,
    pub png_rgba_path: PathBuf,
    pub webp_rgba_path: PathBuf,
    pub heic_path: PathBuf,
}

impl TestFixtures {
    pub fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let dir_path = dir.path();

        // 1. Synthetic JPEG (800x600) saved at high quality (98)
        let jpeg_path = dir_path.join("test_sample.jpg");
        {
            let mut rgb_data = vec![0u8; 800 * 600 * 3];
            for y in 0..600 {
                for x in 0..800 {
                    let offset = (y * 800 + x) * 3;
                    rgb_data[offset] = ((x * 255) / 800) as u8;
                    rgb_data[offset + 1] = ((y * 255) / 600) as u8;
                    rgb_data[offset + 2] = ((x + y) % 256) as u8;
                }
            }
            let mut compressor = turbojpeg::Compressor::new().expect("turbojpeg compressor");
            compressor.set_quality(98).expect("set quality 98");
            let image_ref = turbojpeg::Image {
                pixels: &rgb_data[..],
                width: 800,
                pitch: 800 * 3,
                height: 600,
                format: turbojpeg::PixelFormat::RGB,
            };
            let jpeg_bytes = compressor
                .compress_to_vec(image_ref)
                .expect("compress jpeg");
            std::fs::write(&jpeg_path, jpeg_bytes).expect("write jpeg fixture");
        }

        // 2. Synthetic PNG RGB (400x300)
        let png_rgb_path = dir_path.join("test_rgb.png");
        {
            let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(400, 300);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let r = (x % 256) as u8;
                let g = (y % 256) as u8;
                let b = ((x + y) % 256) as u8;
                *pixel = Rgb([r, g, b]);
            }
            img.save_with_format(&png_rgb_path, image::ImageFormat::Png)
                .expect("failed to save synthetic png rgb");
        }

        // 3. Synthetic PNG RGBA with transparency (400x300)
        // Top half fully transparent (alpha 0) with red background
        // Bottom-left semi-transparent (alpha 128) with green
        // Bottom-right fully opaque (alpha 255) with blue
        let png_rgba_path = dir_path.join("test_transparent.png");
        {
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(400, 300);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                if y < 150 {
                    // Fully transparent (alpha = 0)
                    *pixel = Rgba([255, 0, 0, 0]);
                } else if x < 200 {
                    // Semi-transparent green (alpha = 128)
                    *pixel = Rgba([0, 255, 0, 128]);
                } else {
                    // Fully opaque blue (alpha = 255)
                    *pixel = Rgba([0, 0, 255, 255]);
                }
            }
            img.save_with_format(&png_rgba_path, image::ImageFormat::Png)
                .expect("failed to save synthetic png rgba");
        }

        // 4. Synthetic WebP RGBA (500x500)
        let webp_rgba_path = dir_path.join("test_sample.webp");
        {
            let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(500, 500);
            for (x, y, pixel) in img.enumerate_pixels_mut() {
                let a = if x < 250 { 0 } else { 255 };
                let r = ((x * 255) / 500) as u8;
                let g = ((y * 255) / 500) as u8;
                let b = ((x + y) % 256) as u8;
                *pixel = Rgba([r, g, b, a]);
            }
            // Encode WebP via webp crate at high quality (98) so compression with Balanced (82) saves bytes
            let raw_bytes = img.into_raw();
            let encoder = webp::Encoder::from_rgba(&raw_bytes, 500, 500);
            let webp_data = encoder.encode(98.0);
            std::fs::write(&webp_rgba_path, &*webp_data).expect("failed to save synthetic webp");
        }

        // 5. Synthetic HEIC (320x240)
        let heic_path = dir_path.join("test_sample.heic");
        {
            let width = 320u32;
            let height = 240u32;
            let lib_heif = LibHeif::new();
            let mut image = libheif_rs::Image::new(width, height, ColorSpace::Rgb(RgbChroma::Rgb))
                .expect("failed to create heif image");

            image
                .create_plane(Channel::Interleaved, width, height, 8)
                .expect("failed to create interleaved plane");

            let planes = image.planes_mut();
            let plane = planes.interleaved.expect("missing interleaved plane");
            let stride = plane.stride;
            let data = plane.data;

            for y in 0..height as usize {
                for x in 0..width as usize {
                    let offset = y * stride + x * 3;
                    data[offset] = (x % 256) as u8;
                    data[offset + 1] = (y % 256) as u8;
                    data[offset + 2] = 180;
                }
            }

            let mut context = HeifContext::new().expect("failed to create heif context");
            let mut encoder = lib_heif
                .encoder_for_format(CompressionFormat::Hevc)
                .expect("failed to get hevc encoder");
            let _ = encoder.set_quality(libheif_rs::EncoderQuality::Lossy(98));
            let mut options =
                libheif_rs::EncodingOptions::new().expect("failed to create encoding options");
            options.set_save_alpha_channel(false);

            context
                .encode_image(&image, &mut encoder, Some(options))
                .expect("failed to encode heif image");

            context
                .write_to_file(heic_path.to_str().unwrap())
                .expect("failed to write heif file");
        }

        Self {
            dir,
            jpeg_path,
            png_rgb_path,
            png_rgba_path,
            webp_rgba_path,
            heic_path,
        }
    }
}
