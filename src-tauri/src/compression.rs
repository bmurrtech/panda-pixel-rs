use anyhow::{anyhow, Result};
use image::{self, DynamicImage, ImageFormat};
use imagequant::{Attributes, Image as LiqImage};
use mozjpeg::{ColorSpace, Compress, ScanMode};
use oxipng::{optimize_from_memory, Options as OxipngOptions};
use ravif::{Encoder as AvifEncoder};
use std::io::Cursor;
use webp::Encoder as WebpEncoder;

use crate::compression_options::CompressionOptions;

/// Parse "50-80" into (min,max) u8
pub fn parse_quality_range(s: &str) -> (u8, u8) {
    let parts: Vec<_> = s.split('-').collect();
    let min = parts.get(0).and_then(|p| p.parse::<u8>().ok()).unwrap_or(50);
    let max = parts.get(1).and_then(|p| p.parse::<u8>().ok()).unwrap_or(80);
    (min, max)
}

/// Map compression level (low/mid/max) to quality range
pub fn compression_level_to_range(level: &str) -> String {
    match level.to_lowercase().as_str() {
        "low" => "70-90".to_string(),
        "mid" => "50-80".to_string(),
        "max" => "20-60".to_string(),
        _ => "50-80".to_string(), // Default to mid
    }
}

/// PNG: quantize via libimagequant + optional oxipng (lossless)
pub fn compress_png_bytes(input: &[u8], quality_range: &str, run_oxipng: bool) -> Result<Vec<u8>> {
    // Decode to RGBA8
    let img = image::load_from_memory(input)?;
    let rgba = img.to_rgba8();
    let (w_u32, h_u32) = (rgba.width(), rgba.height());
    let (w, h) = (w_u32 as usize, h_u32 as usize);

    // parse quality
    let (min_q, max_q) = parse_quality_range(quality_range);
    
    // For max compression (20-60 range), use aggressive settings
    let is_max_compression = max_q <= 60;

    // libimagequant
    let mut attr = Attributes::new();
    
    // Adjust speed based on compression level
    if is_max_compression {
        attr.set_speed(1)?; // Slowest, highest quality quantization
        attr.set_max_colors(128)?; // Reduce palette size for max compression
    } else {
        attr.set_speed(3)?; // Balanced speed
    }
    
    attr.set_quality(min_q, max_q)?;
    
    // Convert Vec<u8> to the expected RGBA format
    let rgba_pixels: Vec<rgb::RGBA<u8>> = rgba.chunks_exact(4)
        .map(|chunk| rgb::RGBA::new(chunk[0], chunk[1], chunk[2], chunk[3]))
        .collect();
    
    let mut img_liq = LiqImage::new(&attr, rgba_pixels.as_slice(), w, h, 0.0)?;
    let mut res = attr.quantize(&mut img_liq)?;
    res.set_dithering_level(1.0)?;

    let (palette, pixels) = res.remapped(&mut img_liq)?;

    // Encode as RGBA PNG by expanding palette indices.
    let mut expanded = Vec::with_capacity(w * h * 4);
    for idx in pixels.iter() {
        let p = palette[*idx as usize];
        expanded.push(p.r);
        expanded.push(p.g);
        expanded.push(p.b);
        expanded.push(p.a);
    }

    let dyn_img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(w_u32, h_u32, expanded)
            .ok_or_else(|| anyhow!("failed to build indexed->rgba image"))?,
    );

    let mut cursor = Cursor::new(Vec::new());
    dyn_img.write_to(&mut cursor, ImageFormat::Png)?;
    let png_buf = cursor.into_inner();

    // Optional oxipng optimization (lossless)
    if run_oxipng {
        let mut opts = OxipngOptions::from_preset(6);
        opts.strip = oxipng::StripChunks::Safe;
        let optimized = optimize_from_memory(&png_buf, &opts)?;
        return Ok(optimized);
    }

    Ok(png_buf)
}

/// JPEG: re-encode with mozjpeg
pub fn compress_jpeg_bytes(input: &[u8], quality: u8) -> Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);

    let mut comp = Compress::new(ColorSpace::JCS_RGB);
    comp.set_size(w, h);
    comp.set_quality(quality as f32);
    comp.set_progressive_mode();
    comp.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
    
    // For max compression, enable additional optimization
    if quality <= 60 {
        comp.set_optimize_coding(true);
        comp.set_optimize_scans(true);
    }

    let mut dest = Vec::new();
    let mut writer = comp.start_compress(&mut dest)?;

    // mozjpeg expects raw RGB bytes
    let data = rgb.into_raw();
    writer.write_scanlines(&data)?;
    writer.finish()?;

    Ok(dest)
}

/// WebP via webp crate (lossy) 
pub fn to_webp_bytes(input: &[u8], quality: f32) -> Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    let rgba = img.to_rgba8();
    let enc = WebpEncoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
    let webp = enc.encode(quality); // 0..=100
    Ok(webp.to_vec())
}

/// Convert HEIC to JPEG (like TinyPNG behavior)
pub fn heic_to_jpeg_bytes(input: &[u8], quality: u8) -> Result<Vec<u8>> {
    // Try to decode as HEIC using image crate fallback
    // If image crate doesn't support HEIC, we'll get an error and handle gracefully
    let img = image::load_from_memory(input)
        .map_err(|_| anyhow!("Unsupported HEIC format or corrupted file"))?;
        
    let rgb = img.to_rgb8();
    compress_jpeg_bytes(&{
        let mut cursor = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(rgb).write_to(&mut cursor, ImageFormat::Jpeg)?;
        cursor.into_inner()
    }, quality)
}

/// Convert to PNG
pub fn to_png_bytes(input: &[u8], quality_range: &str, use_oxipng: bool) -> Result<Vec<u8>> {
    // Use PNG compression with quality settings
    compress_png_bytes(input, quality_range, use_oxipng)
}

/// Convert to TIFF
pub fn to_tiff_bytes(input: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, ImageFormat::Tiff)?;
    Ok(cursor.into_inner())
}

/// Convert to BMP
pub fn to_bmp_bytes(input: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, ImageFormat::Bmp)?;
    Ok(cursor.into_inner())
}

/// Convert to ICO (fallback to PNG if ICO not supported)
pub fn to_ico_bytes(input: &[u8]) -> Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    // Resize to common icon size if needed
    let resized = if img.width() > 256 || img.height() > 256 {
        img.resize(256, 256, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    
    let mut cursor = Cursor::new(Vec::new());
    // Try ICO first, fallback to PNG if not supported
    match resized.write_to(&mut cursor, ImageFormat::Ico) {
        Ok(_) => Ok(cursor.into_inner()),
        Err(_) => {
            // Fallback to PNG for ICO
            let mut png_cursor = Cursor::new(Vec::new());
            resized.write_to(&mut png_cursor, ImageFormat::Png)?;
            Ok(png_cursor.into_inner())
        }
    }
}

/// AVIF via ravif crate (lossy)
pub fn to_avif_bytes(input: &[u8], quality: f32) -> Result<Vec<u8>> {
    let img = image::load_from_memory(input)?;
    let rgba = img.to_rgba8();
    let (w, h) = (img.width(), img.height());
    let speed = 6u8; // 0 best / slowest, 10 fastest
    let enc = AvifEncoder::new().with_quality(quality).with_speed(speed);
    
    // Convert to proper RGBA format
    let rgba_pixels: Vec<rgb::RGBA<u8>> = rgba.chunks_exact(4)
        .map(|chunk| rgb::RGBA::new(chunk[0], chunk[1], chunk[2], chunk[3]))
        .collect();
    
    let avif_img = ravif::Img::new(rgba_pixels.as_slice(), w as usize, h as usize);
    let avif = enc.encode_rgba(avif_img)?;
    Ok(avif.avif_file)
}

/// In-process compress dispatcher
pub fn compress_image_inproc(input_bytes: &[u8], ext_lower: &str, opts: &CompressionOptions) -> Result<(Vec<u8>, String)> {
    // Handle HEIC files first (convert to JPEG like TinyPNG)
    if ext_lower == "heic" || ext_lower == "heif" {
        let bytes = heic_to_jpeg_bytes(input_bytes, 85)?; // High quality for HEIC conversion
        return Ok((bytes, "image/jpeg".to_string()));
    }
    
    // Parse quality range to determine compression level
    let (min_q, max_q) = parse_quality_range(&opts.png_quality);
    let webp_quality = ((min_q + max_q) / 2) as f32;
    let jpeg_quality = (min_q + max_q) / 2;
    let avif_quality = ((min_q + max_q) / 2) as f32;
    
    // If conversion requested, honor it next
    if opts.to_webp {
        let bytes = to_webp_bytes(input_bytes, webp_quality)?;
        return Ok((bytes, "image/webp".to_string()));
    }
    if opts.to_avif {
        let bytes = to_avif_bytes(input_bytes, avif_quality)?;
        return Ok((bytes, "image/avif".to_string()));
    }
    if opts.to_jpeg {
        let bytes = compress_jpeg_bytes(input_bytes, jpeg_quality)?;
        return Ok((bytes, "image/jpeg".to_string()));
    }
    if opts.to_png {
        let bytes = to_png_bytes(input_bytes, &opts.png_quality, opts.oxipng)?;
        return Ok((bytes, "image/png".to_string()));
    }
    if opts.to_tiff {
        let bytes = to_tiff_bytes(input_bytes)?;
        return Ok((bytes, "image/tiff".to_string()));
    }
    if opts.to_bmp {
        let bytes = to_bmp_bytes(input_bytes)?;
        return Ok((bytes, "image/bmp".to_string()));
    }
    if opts.to_ico {
        let bytes = to_ico_bytes(input_bytes)?;
        return Ok((bytes, "image/x-icon".to_string()));
    }

    match ext_lower {
        "png" => {
            if opts.png_lossy {
                let bytes = compress_png_bytes(input_bytes, &opts.png_quality, opts.oxipng)?;
                Ok((bytes, "image/png".into()))
            } else {
                // lossless re-encode
                let img = image::load_from_memory(input_bytes)?;
                let mut cursor = Cursor::new(Vec::new());
                img.write_to(&mut cursor, ImageFormat::Png)?;
                let buf = cursor.into_inner();
                Ok((buf, "image/png".into()))
            }
        }
        "jpg" | "jpeg" => {
            let bytes = compress_jpeg_bytes(input_bytes, 75)?;
            Ok((bytes, "image/jpeg".into()))
        }
        // Other formats → PNG by default
        _ => {
            let bytes = compress_png_bytes(input_bytes, &opts.png_quality, opts.oxipng)?;
            Ok((bytes, "image/png".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression_options::CompressionOptions;
    use image::{DynamicImage, ImageFormat};
    use std::io::Cursor;

    fn create_test_png() -> Vec<u8> {
        let img = image::ImageBuffer::from_fn(100, 100, |_, _| {
            image::Rgb([255, 0, 0])
        });
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut bytes = Vec::new();
        dynamic_img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png).unwrap();
        bytes
    }

    fn create_test_jpeg() -> Vec<u8> {
        let img = image::ImageBuffer::from_fn(100, 100, |_, _| {
            image::Rgb([0, 0, 255])
        });
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut bytes = Vec::new();
        dynamic_img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg).unwrap();
        bytes
    }

    #[test]
    fn test_png_compression() {
        let png_data = create_test_png();
        let opts = CompressionOptions {
            png_lossy: true,
            png_quality: "50-80".to_string(),
            oxipng: true,
            ..Default::default()
        };
        
        let result = compress_image_inproc(&png_data, "png", &opts);
        assert!(result.is_ok());
        
        let (compressed, mime_type) = result.unwrap();
        assert_eq!(mime_type, "image/png");
        assert!(compressed.len() > 0);
    }

    #[test]
    fn test_jpeg_compression() {
        let jpeg_data = create_test_jpeg();
        let opts = CompressionOptions::default();
        
        let result = compress_image_inproc(&jpeg_data, "jpeg", &opts);
        assert!(result.is_ok());
        
        let (compressed, mime_type) = result.unwrap();
        assert_eq!(mime_type, "image/jpeg");
        assert!(compressed.len() > 0);
    }

    #[test]
    fn test_quality_parsing() {
        assert_eq!(parse_quality_range("50-80"), (50, 80));
        assert_eq!(parse_quality_range("20-60"), (20, 60));
        assert_eq!(parse_quality_range("70-90"), (70, 90));
    }
}
