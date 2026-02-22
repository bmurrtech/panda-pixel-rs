use crate::state::AppState;
use crate::tauri_helpers;
use crate::utils;
use js_sys;
use leptos::prelude::*;
use serde_json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn CompressButton(state: AppState) -> impl IntoView {
    // Use closure to avoid reactive warnings
    let button_text = move || {
        if state.has_compressed.get() {
            let count = state.results.get().len();
            format!("📥 Download All ({} files)", count)
        } else {
            let count = state.files.get().len();
            format!(
                "Compress & Convert {} Image{}",
                count,
                if count != 1 { "s" } else { "" }
            )
        }
    };

    let show_button = move || !state.files.get().is_empty();

    let compress_files = move |_| {
        // Use get_untracked() in event handler to avoid reactive warnings
        if state.has_compressed.get_untracked() {
            // Download all files - use folder picker approach
            spawn_local(async move {
                let results = state.results.get_untracked();
                if utils::is_dev_mode() {
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                        "📥 Download All: {} files",
                        results.len()
                    )));
                }

                // First, ask user to select output folder
                match tauri_helpers::invoke_tauri::<String>("select_output_folder", JsValue::NULL)
                    .await
                {
                    Ok(output_folder) => {
                        if utils::is_dev_mode() {
                            let folder_name = utils::basename(&output_folder);
                            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                                "✅ Selected folder: {}",
                                folder_name
                            )));
                        }

                        // Prepare files to save
                        let mut files_to_save = Vec::new();
                        for result in results.iter() {
                            let original_path = result.original_path.clone();

                            // Extract just the stem (filename without extension) from the original path
                            let stem = std::path::Path::new(&original_path)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("compressed");

                            // Determine output extension based on mime type
                            let ext = match result.mime_type.as_str() {
                                "image/webp" => "webp",
                                "image/avif" => "avif",
                                "image/jpeg" => "jpg",
                                "image/png" => "png",
                                "image/tiff" => "tiff",
                                "image/bmp" => "bmp",
                                "image/x-icon" => "ico",
                                _ => std::path::Path::new(&original_path)
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("webp"),
                            };

                            // Build filename: stem + new extension (e.g., "image.webp")
                            let filename = format!("{}.{}", stem, ext);

                            // Convert Vec<u8> to JSON array of numbers
                            let data_array: Vec<u8> = result.data.clone();

                            files_to_save.push(serde_json::json!({
                                "filename": filename,
                                "data": data_array,
                            }));
                        }

                        let args_obj = serde_json::json!({
                            "outputFolder": output_folder,
                            "files": files_to_save,
                        });
                        let args = js_sys::JSON::parse(
                            &serde_json::to_string(&args_obj).unwrap_or_default(),
                        )
                        .unwrap_or(JsValue::NULL);

                        if utils::is_dev_mode() {
                            let file_count = files_to_save.len();
                            let sample_names: Vec<String> = files_to_save
                                .iter()
                                .take(3)
                                .filter_map(|f| {
                                    f.get("filename")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                })
                                .collect();
                            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                                "💾 Saving {} files to folder (sample: {:?})",
                                file_count, sample_names
                            )));
                        }

                        match tauri_helpers::invoke_tauri::<Vec<String>>(
                            "save_files_to_folder",
                            args,
                        )
                        .await
                        {
                            Ok(saved_paths) => {
                                if utils::is_dev_mode() {
                                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                                        &format!("✅ Saved {} files", saved_paths.len()),
                                    ));
                                }
                            }
                            Err(e) => {
                                web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(
                                    &format!("❌ Failed to save files: {}", e),
                                ));
                                state.error.set(Some(e));
                            }
                        }
                    }
                    Err(e) => {
                        // Folder selection cancelled or failed
                        if !e.contains("cancelled") && !e.contains("Dialog cancelled") {
                            // Only show error if it's not a user cancellation
                            web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!(
                                "❌ Failed to select folder: {}",
                                e
                            )));
                            state.error.set(Some(e));
                        }
                        // If cancelled, silently return - user can try again
                    }
                }

                // OLD APPROACH - per-file dialogs (kept for reference, but not used)
                /*
                for result in results.iter() {
                    let original_path = result.original_path.clone();

                    // Extract just the stem (filename without extension) from the original path
                    let stem = std::path::Path::new(&original_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("compressed");

                    // Determine output extension based on mime type
                    let ext = match result.mime_type.as_str() {
                        "image/webp" => "webp",
                        "image/avif" => "avif",
                        "image/jpeg" => "jpg",
                        "image/png" => "png",
                        "image/tiff" => "tiff",
                        "image/bmp" => "bmp",
                        "image/x-icon" => "ico",
                        _ => {
                            std::path::Path::new(&original_path)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("webp")
                        }
                    };

                    // Build filename: stem + new extension (e.g., "image.webp")
                    let default_name_with_ext = format!("{}.{}", stem, ext);

                    let args_obj = serde_json::json!({
                        "originalPath": original_path,
                        "defaultName": default_name_with_ext,
                        "data": result.data.clone(),
                    });
                    let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
                            .unwrap_or(JsValue::NULL);

                    if utils::is_dev_mode() {
                        let orig_name = utils::basename(&original_path);
                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("💾 Saving: {} -> {}", orig_name, default_name_with_ext)));
                    }

                    match tauri_helpers::invoke_tauri::<String>("save_file", args).await {
                        Ok(_) => {
                            // Success - silent in release
                        }
                        Err(e) => {
                            // Only log actual errors, not cancellations
                            if !e.contains("cancelled") && !e.contains("Dialog cancelled") {
                                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&format!("⚠️ Save failed: {}", e)));
                            }
                        }
                    }
                }
                */
            });
        } else {
            // Compress files
            spawn_local(async move {
                state.is_compressing.set(true);
                state.progress.set(0.0);
                state.error.set(None);

                // Use get_untracked() in async context to avoid reactive warnings
                let files = state.files.get_untracked();
                let file_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
                let compression_level = state.compression_level.get_untracked();
                let output_format = state.output_format.get_untracked();
                let oxipng = state.oxipng.get_untracked();
                let png_lossy = state.png_lossy.get_untracked();

                let args_obj = serde_json::json!({
                    "filePaths": file_paths,
                    "compressionLevel": compression_level,
                    "outputFormat": output_format,
                    "oxipng": oxipng,
                    "pngLossy": png_lossy,
                });
                let args =
                    js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
                        .unwrap_or(JsValue::NULL);

                if utils::is_dev_mode() {
                    let file_count = file_paths.len();
                    let sample_names: Vec<String> = file_paths
                        .iter()
                        .take(3)
                        .map(|p| utils::basename(p))
                        .collect();
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                        "🔧 compress_batch: {} files, format={}, level={}, sample={:?}",
                        file_count, output_format, compression_level, sample_names
                    )));
                }

                match tauri_helpers::invoke_tauri::<Vec<crate::state::CompressionResult>>(
                    "compress_batch",
                    args,
                )
                .await
                {
                    Ok(results) => {
                        state.results.set(results);
                        state.has_compressed.set(true);
                        state.progress.set(100.0);
                    }
                    Err(e) => {
                        state.error.set(Some(e));
                    }
                }

                state.is_compressing.set(false);
            });
        }
    };

    view! {
        <div class="convert-section">
            <Show when=show_button>
                <button
                    type="button"
                    class="convert-button show"
                    disabled=move || state.is_compressing.get()
                    on:click=compress_files
                >
                    {button_text}
                </button>
            </Show>
        </div>
    }
}
