use compression::compress_image_inproc;
use domain::{compression_level_to_range, CompressionOptions};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::Window;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original_path: String,
    pub compressed_path: Option<String>,
    pub original_size: u64,
    pub compressed_size: u64,
    pub savings_percent: f64,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
}

/// Select files using Tauri file dialog
#[tauri::command]
pub async fn select_files(
    app: tauri::AppHandle,
) -> Result<Vec<FileInfo>, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    
    app.dialog()
        .file()
        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "webp", "heic", "heif", "avif"])
        .pick_files(move |paths| {
            let _ = tx.send(paths);
        });

    let file_paths = rx.recv()
        .map_err(|_| "Dialog cancelled".to_string())?
        .ok_or_else(|| "No files selected".to_string())?;

    let mut files = Vec::new();
    for path in file_paths {
        let path_buf = path.as_path()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_path_buf();
        let metadata = fs::metadata(&path_buf)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        files.push(FileInfo {
            path: path_buf.to_string_lossy().to_string(),
            name,
            size: metadata.len(),
        });
    }

    Ok(files)
}

/// Handle files dropped onto the window (from drag & drop)
#[tauri::command]
pub async fn handle_dropped_files(
    file_paths: Vec<String>,
) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    
    for path_str in file_paths {
        let path_buf = Path::new(&path_str);
        let metadata = fs::metadata(path_buf)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        let name = path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        files.push(FileInfo {
            path: path_str,
            name,
            size: metadata.len(),
        });
    }
    
    Ok(files)
}

/// Compress a single image file
#[tauri::command]
pub async fn compress_image(
    file_path: String,
    compression_level: String,
    output_format: String,
    oxipng: bool,
    png_lossy: bool,
) -> Result<CompressionResult, String> {
    // Read file
    let file_bytes = fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Get file extension
    let ext = Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Map compression level to quality range
    let quality_range = compression_level_to_range(&compression_level);

    // Build compression options
    let mut opts = CompressionOptions {
        png_quality: quality_range,
        oxipng,
        png_lossy,
        ..Default::default()
    };

    // Set output format
    match output_format.as_str() {
        "webp" => opts.to_webp = true,
        "avif" => opts.to_avif = true,
        "jpeg" => opts.to_jpeg = true,
        "png" => opts.to_png = true,
        "tiff" => opts.to_tiff = true,
        "bmp" => opts.to_bmp = true,
        "ico" => opts.to_ico = true,
        "original" => {}, // Keep original format
        _ => opts.to_webp = true, // Default to WebP
    }

    // Compress
    let (compressed_bytes, mime_type) = compress_image_inproc(&file_bytes, &ext, &opts)
        .map_err(|e| format!("Compression failed: {}", e))?;

    let original_size = file_bytes.len() as u64;
    let compressed_size = compressed_bytes.len() as u64;
    let savings_percent = if original_size > 0 {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(CompressionResult {
        original_path: file_path,
        compressed_path: None,
        original_size,
        compressed_size,
        savings_percent,
        mime_type,
        data: compressed_bytes,
    })
}

/// Compress multiple images in batch
#[tauri::command]
pub async fn compress_batch(
    file_paths: Vec<String>,
    compression_level: String,
    output_format: String,
    oxipng: bool,
    png_lossy: bool,
) -> Result<Vec<CompressionResult>, String> {
    let mut results = Vec::new();

    for file_path in file_paths {
        match compress_image(
            file_path.clone(),
            compression_level.clone(),
            output_format.clone(),
            oxipng,
            png_lossy,
        )
        .await
        {
            Ok(result) => results.push(result),
            Err(e) => {
                // Continue with other files even if one fails
                let file_path_clone = file_path.clone();
                results.push(CompressionResult {
                    original_path: file_path_clone.clone(),
                    compressed_path: None,
                    original_size: 0,
                    compressed_size: 0,
                    savings_percent: 0.0,
                    mime_type: String::new(),
                    data: Vec::new(),
                });
                log::error!("Failed to compress {}: {}", file_path_clone, e);
            }
        }
    }

    Ok(results)
}

/// Select output folder for saving files
#[tauri::command]
pub async fn select_output_folder(
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    
    app.dialog()
        .file()
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });

    let folder_path = rx.recv()
        .map_err(|_| "Dialog cancelled".to_string())?
        .ok_or_else(|| "No folder selected".to_string())?;

    let path_buf = folder_path.as_path()
        .ok_or_else(|| "Invalid path".to_string())?
        .to_path_buf();
    
    Ok(path_buf.to_string_lossy().to_string())
}

/// Save multiple files to a folder (no per-file dialogs)
#[tauri::command]
pub async fn save_files_to_folder(
    output_folder: String,
    files: Vec<serde_json::Value>,
) -> Result<Vec<String>, String> {
    use std::path::Path;
    use std::fs;
    use serde_json;

    let folder_path = Path::new(&output_folder);
    
    // Ensure folder exists
    fs::create_dir_all(folder_path)
        .map_err(|e| format!("Failed to create folder: {}", e))?;

    let mut saved_paths = Vec::new();

    for file in files {
        let filename_str = file.get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing filename".to_string())?;
        
        // Sanitize filename to prevent path traversal
        let filename = Path::new(filename_str)
            .file_name()
            .ok_or_else(|| "Invalid filename".to_string())?
            .to_string_lossy();
        
        // Parse data array from JSON - it comes as a JSON array of numbers
        let data_array = file.get("data")
            .ok_or_else(|| "Missing data".to_string())?;
        
        // Convert JSON array to Vec<u8>
        let data: Vec<u8> = serde_json::from_value(data_array.clone())
            .map_err(|e| format!("Failed to parse data for {}: {}", filename, e))?;

        let file_path = folder_path.join(filename.as_ref());
        let data_len = data.len(); // Store length before move
        
        fs::write(&file_path, data)
            .map_err(|e| format!("Failed to save {}: {}", filename, e))?;
        
        saved_paths.push(file_path.to_string_lossy().to_string());
        
        #[cfg(debug_assertions)]
        {
            log::debug!("Saved: {} ({} bytes)", filename, data_len);
        }
    }

    Ok(saved_paths)
}

/// Save compressed file to disk (saves in same directory as source by default)
#[tauri::command]
pub async fn save_file(
    app: tauri::AppHandle,
    original_path: String,
    default_name: String,
    data: Vec<u8>,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    use std::path::Path;

    let (tx, rx) = mpsc::channel();
    
    // Extract just the filename from default_name (in case it contains a path)
    // default_name should already be just the filename (e.g., "image.webp"), but be safe
    let file_name_only = Path::new(&default_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&default_name)
        .to_string();
    
    // Log summary only in debug mode
    #[cfg(debug_assertions)]
    {
        use std::path::Path;
        let orig_name = Path::new(&original_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        log::debug!("Save dialog: {} -> {}", orig_name, file_name_only);
    }
    
    // Pass only the filename to set_file_name - this ensures the dialog shows
    // just the filename, not the full path. The dialog will open in the user's
    // default save location (or last used directory).
    app.dialog()
        .file()
        .set_file_name(&file_name_only) // Only filename, not full path
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let save_path = rx.recv()
        .map_err(|_| "Dialog cancelled".to_string())?
        .ok_or_else(|| "Save cancelled".to_string())?;

    let path_buf = save_path.as_path()
        .ok_or_else(|| "Invalid path".to_string())?
        .to_path_buf();
    fs::write(&path_buf, data)
        .map_err(|e| format!("Failed to save file: {}", e))?;

    Ok(path_buf.to_string_lossy().to_string())
}

/// Resize window to fit content
#[tauri::command]
pub async fn resize_window(
    window: Window,
    width: f64,
    height: f64,
) -> Result<(), String> {
    // Add padding for window chrome (title bar, borders, etc.)
    let padding = 40.0;
    window
        .set_size(tauri::LogicalSize::new(width + padding, height + padding))
        .map_err(|e| format!("Failed to resize window: {}", e))?;
    Ok(())
}

/// Open devtools (for debugging)
#[tauri::command]
pub async fn open_devtools(app: tauri::AppHandle) -> Result<(), String> {
    // In Tauri 2.0, devtools are opened via the window's webview
    #[cfg(debug_assertions)]
    {
        use tauri::Manager;
        if let Some(webview) = app.get_webview_window("main") {
            webview.open_devtools();
        }
    }
    Ok(())
}
