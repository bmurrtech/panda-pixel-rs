use serde::{Deserialize, Serialize};

/// Compression options for image processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOptions {
    pub png_lossy: bool,
    pub png_quality: String,
    pub oxipng: bool,
    pub to_webp: bool,
    pub to_avif: bool,
    pub to_jpeg: bool,
    pub to_png: bool,
    pub to_tiff: bool,
    pub to_bmp: bool,
    pub to_ico: bool,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            png_lossy: true,
            png_quality: "50-80".to_string(),
            oxipng: true,
            to_webp: false,
            to_avif: false,
            to_jpeg: false,
            to_png: false,
            to_tiff: false,
            to_bmp: false,
            to_ico: false,
        }
    }
}

/// Parse "50-80" into (min,max) u8
pub fn parse_quality_range(s: &str) -> (u8, u8) {
    let parts: Vec<_> = s.split('-').collect();
    let min = parts.first().and_then(|p| p.parse::<u8>().ok()).unwrap_or(50);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_parsing() {
        assert_eq!(parse_quality_range("50-80"), (50, 80));
        assert_eq!(parse_quality_range("20-60"), (20, 60));
        assert_eq!(parse_quality_range("70-90"), (70, 90));
    }

    #[test]
    fn test_compression_level_to_range() {
        assert_eq!(compression_level_to_range("low"), "70-90");
        assert_eq!(compression_level_to_range("mid"), "50-80");
        assert_eq!(compression_level_to_range("max"), "20-60");
        assert_eq!(compression_level_to_range("unknown"), "50-80");
    }
}
