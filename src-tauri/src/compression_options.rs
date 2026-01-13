use serde::{Deserialize, Serialize};

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
