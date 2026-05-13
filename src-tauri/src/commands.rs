use crate::compression::{compress_image_inproc, compression_level_to_range};
use crate::compression_options::CompressionOptions;
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
    #[serde(default)]
    pub requested_output_format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
}

/// Select files using Tauri file dialog
#[tauri::command]
pub async fn select_files(app: tauri::AppHandle) -> Result<Vec<FileInfo>, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = mpsc::channel();

    app.dialog()
        .file()
        .add_filter(
            "Images",
            &["png", "jpg", "jpeg", "bmp", "tiff", "tif", "webp", "ico"],
        )
        .pick_files(move |paths| {
            let _ = tx.send(paths);
        });

    let file_paths = rx
        .recv()
        .map_err(|_| "Dialog cancelled".to_string())?
        .ok_or_else(|| "No files selected".to_string())?;

    let mut files = Vec::new();
    for path in file_paths {
        let path_buf = path
            .as_path()
            .ok_or_else(|| "Invalid path".to_string())?
            .to_path_buf();
        let metadata =
            fs::metadata(&path_buf).map_err(|e| format!("Failed to read file metadata: {}", e))?;
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
pub async fn handle_dropped_files(file_paths: Vec<String>) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();

    for path_str in file_paths {
        let path_buf = Path::new(&path_str);
        let metadata =
            fs::metadata(path_buf).map_err(|e| format!("Failed to read file metadata: {}", e))?;
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
    let file_bytes = fs::read(&file_path).map_err(|e| format!("Failed to read file: {}", e))?;

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
        "original" => {}          // Keep original format
        _ => opts.to_webp = true, // Default to WebP
    }

    // Compress
    let (compressed_bytes, mime_type) = compress_image_inproc(&file_bytes, &ext, &opts)
        .map_err(|e| format!("Compression failed: {}", e))?;

    let original_size = file_bytes.len() as u64;
    let compressed_size = compressed_bytes.len() as u64;
    let savings_percent = if original_size > 0 {
        ((compressed_size as f64 - original_size as f64) / original_size as f64) * 100.0
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
        requested_output_format: Some(output_format),
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
                    requested_output_format: None,
                });
                log::error!("Failed to compress {}: {}", file_path_clone, e);
            }
        }
    }

    Ok(results)
}

/// Select output folder for saving files
#[tauri::command]
pub async fn select_output_folder(app: tauri::AppHandle) -> Result<String, String> {
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = mpsc::channel();

    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path);
    });

    let folder_path = rx
        .recv()
        .map_err(|_| "Dialog cancelled".to_string())?
        .ok_or_else(|| "No folder selected".to_string())?;

    let path_buf = folder_path
        .as_path()
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
    use serde_json;
    use std::fs;
    use std::path::Path;

    let folder_path = Path::new(&output_folder);

    // Ensure folder exists
    fs::create_dir_all(folder_path).map_err(|e| format!("Failed to create folder: {}", e))?;

    let mut saved_paths = Vec::new();

    for file in files {
        let filename = file
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing filename".to_string())?;

        // Parse data array from JSON - it comes as a JSON array of numbers
        let data_array = file.get("data").ok_or_else(|| "Missing data".to_string())?;

        // Convert JSON array to Vec<u8>
        let data: Vec<u8> = serde_json::from_value(data_array.clone())
            .map_err(|e| format!("Failed to parse data for {}: {}", filename, e))?;

        let file_path = folder_path.join(filename);

        fs::write(&file_path, data).map_err(|e| format!("Failed to save {}: {}", filename, e))?;

        saved_paths.push(file_path.to_string_lossy().to_string());

        #[cfg(debug_assertions)]
        {
            let data_len = file
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len())
                .unwrap_or_default();
            log::debug!("Saved: {} ({} bytes)", filename, data_len);
        }
    }

    Ok(saved_paths)
}

#[tauri::command]
pub async fn check_file_collisions(
    output_folder: String,
    filenames: Vec<String>,
) -> Result<Vec<String>, String> {
    use std::path::Path;

    let folder_path = Path::new(&output_folder);
    let mut collisions = Vec::new();

    for filename in filenames {
        let file_path = folder_path.join(&filename);
        if file_path.exists() {
            collisions.push(filename);
        }
    }

    Ok(collisions)
}

/// Returns display filenames that are unique in `output_folder` and within the batch (order-preserving).
#[tauri::command]
pub async fn resolve_unique_filenames(
    output_folder: String,
    filenames: Vec<String>,
) -> Result<Vec<String>, String> {
    let folder_path = Path::new(&output_folder);
    crate::filename_unique::resolve_unique_names_for_disk(folder_path, &filenames)
}

#[tauri::command]
pub async fn save_files_as_zip(
    output_folder: String,
    zip_filename: String,
    files: Vec<serde_json::Value>,
) -> Result<String, String> {
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use zip::write::SimpleFileOptions;
    use zip::CompressionMethod;
    use zip::ZipWriter;

    let folder_path = Path::new(&output_folder);
    fs::create_dir_all(folder_path).map_err(|e| format!("Failed to create folder: {}", e))?;

    let zip_path = folder_path.join(&zip_filename);
    let zip_file =
        File::create(&zip_path).map_err(|e| format!("Failed to create ZIP file: {}", e))?;

    let mut zip = ZipWriter::new(zip_file);
    let file_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(6));

    let names: Vec<String> = files
        .iter()
        .map(|file| {
            file.get("filename")
                .and_then(|v| v.as_str())
                .map(String::from)
                .ok_or_else(|| "Missing filename".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let unique_names = crate::filename_unique::uniquify_zip_entry_names(&names)?;

    for (file, filename) in files.iter().zip(unique_names.iter()) {
        let data_array = file
            .get("data")
            .ok_or_else(|| format!("Missing data for {}", filename))?;

        let data: Vec<u8> = serde_json::from_value(data_array.clone())
            .map_err(|e| format!("Failed to parse data for {}: {}", filename, e))?;

        zip.start_file(filename, file_options)
            .map_err(|e| format!("Failed to add {} to ZIP: {}", filename, e))?;

        zip.write_all(&data)
            .map_err(|e| format!("Failed to write {} to ZIP: {}", filename, e))?;
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP archive: {}", e))?;

    Ok(zip_path.to_string_lossy().to_string())
}

/// Save compressed file to disk (saves in same directory as source by default)
#[tauri::command]
pub async fn save_file(
    app: tauri::AppHandle,
    original_path: String,
    default_name: String,
    data: Vec<u8>,
) -> Result<String, String> {
    use std::path::Path;
    use std::sync::mpsc;
    use tauri_plugin_dialog::DialogExt;

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
    #[cfg(not(debug_assertions))]
    let _ = &original_path;

    // Pass only the filename to set_file_name - this ensures the dialog shows
    // just the filename, not the full path. The dialog will open in the user's
    // default save location (or last used directory).
    app.dialog()
        .file()
        .set_file_name(&file_name_only) // Only filename, not full path
        .save_file(move |path| {
            let _ = tx.send(path);
        });

    let save_path = rx
        .recv()
        .map_err(|_| "Dialog cancelled".to_string())?
        .ok_or_else(|| "Save cancelled".to_string())?;

    let path_buf = save_path
        .as_path()
        .ok_or_else(|| "Invalid path".to_string())?
        .to_path_buf();
    fs::write(&path_buf, data).map_err(|e| format!("Failed to save file: {}", e))?;

    Ok(path_buf.to_string_lossy().to_string())
}

/// Resize window to fit content
#[tauri::command]
pub async fn resize_window(window: Window, width: f64, height: f64) -> Result<(), String> {
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
    #[cfg(not(debug_assertions))]
    let _ = app;
    Ok(())
}

/// Open a folder path in the native file manager.
#[tauri::command]
pub async fn open_in_file_manager(path: String) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(path);
        c
    };

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = Command::new("explorer");
        c.arg(path);
        c
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(path);
        c
    };

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to open folder in file manager: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("File manager exited with status: {status}"))
    }
}
