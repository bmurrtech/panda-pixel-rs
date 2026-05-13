use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

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
    /// UI output format at compress time (`webp`, `jpeg`, `original`, …). Drives display/download extensions.
    #[serde(default)]
    pub requested_output_format: Option<String>,
}

impl CompressionResult {
    /// Filename (stem + extension) shown to the user and used for downloads.
    pub fn display_export_filename(&self) -> String {
        let path = Path::new(&self.original_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("compressed");
        let ext = self.export_extension();
        format!("{stem}.{ext}")
    }

    fn export_extension(&self) -> String {
        if let Some(ref fmt) = self.requested_output_format {
            let f = fmt.trim().to_lowercase();
            return match f.as_str() {
                "original" => self.ext_from_mime_or_path(),
                "webp" => "webp".to_string(),
                "avif" => "avif".to_string(),
                "jpeg" => "jpg".to_string(),
                "png" => "png".to_string(),
                "tiff" => "tiff".to_string(),
                "bmp" => "bmp".to_string(),
                "ico" => "ico".to_string(),
                _ => self.ext_from_mime_or_path(),
            };
        }
        self.ext_from_mime_or_path()
    }

    fn ext_from_mime_or_path(&self) -> String {
        ext_from_mime_type(&self.mime_type).unwrap_or_else(|| {
            Path::new(&self.original_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin")
                .to_string()
        })
    }
}

fn ext_from_mime_type(mime: &str) -> Option<String> {
    match mime {
        "image/webp" => Some("webp".into()),
        "image/avif" => Some("avif".into()),
        "image/jpeg" => Some("jpg".into()),
        "image/png" => Some("png".into()),
        "image/tiff" => Some("tiff".into()),
        "image/bmp" => Some("bmp".into()),
        "image/x-icon" => Some("ico".into()),
        _ => None,
    }
}

/// Pending save options stored in AppState for collision modal access
#[derive(Clone, Debug)]
pub struct PendingSaveOptions {
    pub overwrite: bool,
    pub auto_rename: bool,
    pub as_zip: bool,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub files: RwSignal<Vec<FileInfo>>,
    pub compression_level: RwSignal<String>,
    pub output_format: RwSignal<String>,
    pub oxipng: RwSignal<bool>,
    pub png_lossy: RwSignal<bool>,
    pub progress: RwSignal<f64>,
    pub results: RwSignal<Vec<CompressionResult>>,
    pub error: RwSignal<Option<String>>,
    pub status: RwSignal<Option<String>>,
    pub is_compressing: RwSignal<bool>,
    pub has_compressed: RwSignal<bool>,
    pub advanced_open: RwSignal<bool>,
    pub show_collision_modal: RwSignal<bool>,
    pub collision_files: RwSignal<Vec<String>>,
    /// Editable collision names - mirrors collision_files but user can edit
    pub collision_name_edits: RwSignal<Vec<String>>,
    /// Snapshot of original collision names to detect "dirty" state
    pub collision_initial_snapshot: RwSignal<Vec<String>>,
    pub pending_save_folder: RwSignal<Option<String>>,
    pub pending_save_options: RwSignal<Option<PendingSaveOptions>>,
    pub show_files_list: RwSignal<bool>,
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
            status: RwSignal::new(None),
            is_compressing: RwSignal::new(false),
            has_compressed: RwSignal::new(false),
            advanced_open: RwSignal::new(false),
            show_collision_modal: RwSignal::new(false),
            collision_files: RwSignal::new(Vec::new()),
            collision_name_edits: RwSignal::new(Vec::new()),
            collision_initial_snapshot: RwSignal::new(Vec::new()),
            pending_save_folder: RwSignal::new(None),
            pending_save_options: RwSignal::new(None),
            show_files_list: RwSignal::new(true),
        }
    }

    /// Reset collision modal state when modal closes
    pub fn reset_collision_state(&self) {
        self.show_collision_modal.set(false);
        self.pending_save_folder.set(None);
        self.pending_save_options.set(None);
        self.collision_files.set(Vec::new());
        self.collision_name_edits.set(Vec::new());
        self.collision_initial_snapshot.set(Vec::new());
    }

    /// Initialize collision state when modal opens
    pub fn init_collision_state(
        &self,
        collisions: Vec<String>,
        folder: String,
        options: PendingSaveOptions,
    ) {
        let snapshot = collisions.clone();
        self.collision_files.set(collisions.clone());
        self.collision_name_edits.set(collisions);
        self.collision_initial_snapshot.set(snapshot);
        self.pending_save_folder.set(Some(folder));
        self.pending_save_options.set(Some(options));
        self.show_collision_modal.set(true);
    }
}
