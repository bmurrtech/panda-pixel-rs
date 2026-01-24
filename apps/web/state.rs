use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original_path: String,
    pub compressed_path: Option<String>,
    pub original_size: u64,
    pub compressed_size: u64,
    pub savings_percent: f64,
    pub mime_type: String,
    pub data: Vec<u8>,
}

pub struct AppState {
    pub files: RwSignal<Vec<FileInfo>>,
    pub compression_level: RwSignal<String>,
    pub output_format: RwSignal<String>,
    pub oxipng: RwSignal<bool>,
    pub png_lossy: RwSignal<bool>,
    pub progress: RwSignal<f64>,
    pub results: RwSignal<Vec<CompressionResult>>,
    pub error: RwSignal<Option<String>>,
    pub is_compressing: RwSignal<bool>,
    pub has_compressed: RwSignal<bool>,
    pub advanced_open: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            files: RwSignal::new(Vec::new()),
            compression_level: RwSignal::new("mid".to_string()),
            output_format: RwSignal::new("webp".to_string()),
            oxipng: RwSignal::new(true),
            png_lossy: RwSignal::new(true),
            progress: RwSignal::new(0.0),
            results: RwSignal::new(Vec::new()),
            error: RwSignal::new(None),
            is_compressing: RwSignal::new(false),
            has_compressed: RwSignal::new(false),
            advanced_open: RwSignal::new(false),
        }
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            files: self.files,
            compression_level: self.compression_level,
            output_format: self.output_format,
            oxipng: self.oxipng,
            png_lossy: self.png_lossy,
            progress: self.progress,
            results: self.results,
            error: self.error,
            is_compressing: self.is_compressing,
            has_compressed: self.has_compressed,
            advanced_open: self.advanced_open,
        }
    }
}
