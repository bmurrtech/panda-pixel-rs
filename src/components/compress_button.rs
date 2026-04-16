use leptos::prelude::*;
use crate::state::{AppState, CompressionResult};
use crate::tauri_helpers;
use crate::utils;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys;
use serde_json;

#[derive(Clone)]
struct SaveOptions {
    overwrite: bool,
    auto_rename: bool,
    as_zip: bool,
}

fn generate_filename_from_result(result: &CompressionResult) -> String {
    let original_path = result.original_path.clone();
    let stem = std::path::Path::new(&original_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("compressed");

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

    format!("{}.{}", stem, ext)
}

fn generate_unique_filename(folder: &str, original: &str, existing: &[String]) -> String {
    let full_path = std::path::Path::new(folder).join(original);
    let is_taken = existing.iter().any(|name| name == original) || full_path.exists();

    if !is_taken {
        return original.to_string();
    }

    let original_path = std::path::Path::new(original);
    let stem = original_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("compressed");
    let ext = original_path.extension().and_then(|e| e.to_str());

    for index in 1..10_000 {
        let candidate = match ext {
            Some(ext) => format!("{} ({}).{}", stem, index, ext),
            None => format!("{} ({})", stem, index),
        };

        let candidate_path = std::path::Path::new(folder).join(&candidate);
        let candidate_taken = existing.iter().any(|name| name == &candidate) || candidate_path.exists();

        if !candidate_taken {
            return candidate;
        }
    }

    original.to_string()
}

fn prepare_files_payload(
    results: &[CompressionResult],
    output_folder: &str,
    auto_rename: bool,
    existing: &[String],
) -> Vec<serde_json::Value> {
    let mut files_to_save = Vec::new();
    let mut taken_names = existing.to_vec();

    for result in results {
        let initial_filename = generate_filename_from_result(result);
        let filename = if auto_rename {
            generate_unique_filename(output_folder, &initial_filename, &taken_names)
        } else {
            initial_filename
        };

        taken_names.push(filename.clone());

        files_to_save.push(serde_json::json!({
            "filename": filename,
            "data": result.data.clone(),
        }));
    }

    files_to_save
}

fn build_zip_filename() -> String {
    let now = js_sys::Date::new_0();
    let hours = now.get_hours();
    let minutes = now.get_minutes();
    let seconds = now.get_seconds();
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();

    format!(
        "{hours:02}-{minutes:02}-{seconds:02}-{year:04}-{month:02}-{day:02}-panda-pixel-app.zip"
    )
}

fn to_js_args(value: serde_json::Value) -> JsValue {
    js_sys::JSON::parse(&serde_json::to_string(&value).unwrap_or_default())
        .unwrap_or_else(|_| JsValue::NULL)
}

async fn save_with_options(
    state: AppState,
    pending_save_options: RwSignal<Option<SaveOptions>>,
    options: SaveOptions,
    folder_override: Option<String>,
) {
    let results = state.results.get_untracked();

    if results.is_empty() {
        state.error.set(Some("No compressed files to save".to_string()));
        state.status.set(None);
        return;
    }

    let output_folder = match folder_override {
        Some(folder) => folder,
        None => match tauri_helpers::invoke_tauri::<String>("select_output_folder", JsValue::NULL).await {
            Ok(folder) => folder,
            Err(e) => {
                if !e.contains("cancelled") && !e.contains("Dialog cancelled") && !e.contains("No folder selected") {
                    state.status.set(None);
                    state.error.set(Some(e));
                }
                return;
            }
        },
    };

    let base_zip_filename = if options.as_zip {
        Some(build_zip_filename())
    } else {
        None
    };
    let collision_targets = if let Some(zip_filename) = &base_zip_filename {
        vec![zip_filename.clone()]
    } else {
        results.iter().map(generate_filename_from_result).collect::<Vec<String>>()
    };

    let collision_args = to_js_args(serde_json::json!({
        "outputFolder": output_folder.clone(),
        "filenames": collision_targets,
    }));

    let collisions = match tauri_helpers::invoke_tauri::<Vec<String>>("check_file_collisions", collision_args).await {
        Ok(collisions) => collisions,
        Err(e) => {
            state.status.set(None);
            state.error.set(Some(e));
            return;
        }
    };

    if !collisions.is_empty() && !options.overwrite && !options.auto_rename {
        state.collision_files.set(collisions);
        state.pending_save_folder.set(Some(output_folder));
        pending_save_options.set(Some(options));
        state.show_collision_modal.set(true);
        return;
    }

    state.show_collision_modal.set(false);
    state.pending_save_folder.set(None);
    state.collision_files.set(Vec::new());
    pending_save_options.set(None);

    let files_to_save = prepare_files_payload(&results, &output_folder, options.auto_rename, &collisions);

    if options.as_zip {
        let base_zip_filename = base_zip_filename.unwrap_or_else(build_zip_filename);
        let zip_filename = if options.auto_rename {
            generate_unique_filename(&output_folder, &base_zip_filename, &collisions)
        } else {
            base_zip_filename
        };

        let save_args = to_js_args(serde_json::json!({
            "outputFolder": output_folder.clone(),
            "zipFilename": zip_filename,
            "files": files_to_save,
        }));

        match tauri_helpers::invoke_tauri::<String>("save_files_as_zip", save_args).await {
            Ok(saved_zip_path) => {
                state.error.set(None);
                state.status.set(Some(format!("Saved ZIP: {}", utils::basename(&saved_zip_path))));

                let open_args = to_js_args(serde_json::json!({
                    "path": output_folder.clone(),
                }));
                let _ = tauri_helpers::invoke_tauri::<()>("open_in_file_manager", open_args).await;
            }
            Err(e) => {
                state.status.set(None);
                state.error.set(Some(e));
            }
        }

        return;
    }

    let save_args = to_js_args(serde_json::json!({
        "outputFolder": output_folder.clone(),
        "files": files_to_save,
    }));

    match tauri_helpers::invoke_tauri::<Vec<String>>("save_files_to_folder", save_args).await {
        Ok(saved_paths) => {
            state.error.set(None);
            state.status.set(Some(format!(
                "Saved {} file(s) to {}",
                saved_paths.len(),
                utils::basename(&output_folder)
            )));

            let open_args = to_js_args(serde_json::json!({
                "path": output_folder.clone(),
            }));
            let _ = tauri_helpers::invoke_tauri::<()>("open_in_file_manager", open_args).await;
        }
        Err(e) => {
            state.status.set(None);
            state.error.set(Some(e));
        }
    }
}

fn spawn_save_with_options(
    state: AppState,
    pending_save_options: RwSignal<Option<SaveOptions>>,
    options: SaveOptions,
    folder_override: Option<String>,
) {
    spawn_local(async move {
        save_with_options(state, pending_save_options, options, folder_override).await;
    });
}

#[component]
pub fn CompressButton(state: AppState) -> impl IntoView {
    let button_text = move || {
        let count = state.files.get().len();
        format!(
            "Compress & Convert {} Image{}",
            count,
            if count != 1 { "s" } else { "" }
        )
    };

    let show_button = move || !state.files.get().is_empty();
    let has_multiple_results = move || state.results.get().len() > 1;
    let save_label = move || {
        if has_multiple_results() {
            "Save Files"
        } else {
            "Save File"
        }
    };
    let pending_save_options = RwSignal::new(None::<SaveOptions>);

    let compress_files = move |_| {
        spawn_local(async move {
            state.is_compressing.set(true);
            state.progress.set(0.0);
            state.error.set(None);
            state.status.set(None);

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
            let args = to_js_args(args_obj);

            if utils::is_dev_mode() {
                let file_count = file_paths.len();
                let sample_names: Vec<String> = file_paths.iter()
                    .take(3)
                    .map(|p| utils::basename(p))
                    .collect();
                web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "🔧 compress_batch: {} files, format={}, level={}, sample={:?}",
                    file_count, output_format, compression_level, sample_names
                )));
            }

            match tauri_helpers::invoke_tauri::<Vec<crate::state::CompressionResult>>("compress_batch", args).await {
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
    };

    view! {
        <>
            <div class="convert-section">
                <Show when=show_button>
                    <Show
                        when=move || state.has_compressed.get()
                        fallback=move || {
                            view! {
                                <button
                                    type="button"
                                    class="convert-button show"
                                    disabled=move || state.is_compressing.get()
                                    on:click=compress_files
                                >
                                    {button_text}
                                </button>
                            }
                        }
                    >
                        <div class="save-actions">
                            <button
                                type="button"
                                class="convert-button show save-primary-button"
                                disabled=move || state.is_compressing.get()
                                on:click=move |_| {
                                    spawn_save_with_options(
                                        state.clone(),
                                        pending_save_options,
                                        SaveOptions {
                                            overwrite: false,
                                            auto_rename: false,
                                            as_zip: false,
                                        },
                                        None,
                                    );
                                }
                            >
                                {save_label}
                            </button>
                            <Show when=has_multiple_results>
                                <button
                                    type="button"
                                    class="convert-button show save-zip-button"
                                    disabled=move || state.is_compressing.get()
                                    on:click=move |_| {
                                        spawn_save_with_options(
                                            state.clone(),
                                            pending_save_options,
                                            SaveOptions {
                                                overwrite: false,
                                                auto_rename: false,
                                                as_zip: true,
                                            },
                                            None,
                                        );
                                    }
                                >
                                    "Save as ZIP"
                                </button>
                            </Show>
                        </div>
                    </Show>
                </Show>
            </div>

            <Show when=move || state.show_collision_modal.get()>
                <div class="modal-overlay">
                    <div class="modal-content">
                        <h2>"File Collision Detected"</h2>
                        <p>"The following files already exist in the selected folder:"</p>
                        <ul class="collision-list">
                            <For
                                each=move || state.collision_files.get()
                                key=|filename| filename.clone()
                                children=move |filename| {
                                    view! { <li>{filename}</li> }
                                }
                            />
                        </ul>
                        <p>"Would you like to overwrite them or automatically rename?"</p>
                        <div class="modal-actions">
                            <button
                                type="button"
                                class="btn-secondary"
                                on:click=move |_| {
                                    state.show_collision_modal.set(false);
                                    state.pending_save_folder.set(None);
                                    state.collision_files.set(Vec::new());
                                    pending_save_options.set(None);
                                }
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="btn-primary"
                                on:click=move |_| {
                                    state.show_collision_modal.set(false);

                                    if let (Some(folder), Some(previous_options)) = (
                                        state.pending_save_folder.get_untracked(),
                                        pending_save_options.get_untracked(),
                                    ) {
                                        spawn_save_with_options(
                                            state.clone(),
                                            pending_save_options,
                                            SaveOptions {
                                                overwrite: false,
                                                auto_rename: true,
                                                as_zip: previous_options.as_zip,
                                            },
                                            Some(folder),
                                        );
                                    }
                                }
                            >
                                "Rename"
                            </button>
                            <button
                                type="button"
                                class="btn-danger"
                                on:click=move |_| {
                                    state.show_collision_modal.set(false);

                                    if let (Some(folder), Some(previous_options)) = (
                                        state.pending_save_folder.get_untracked(),
                                        pending_save_options.get_untracked(),
                                    ) {
                                        spawn_save_with_options(
                                            state.clone(),
                                            pending_save_options,
                                            SaveOptions {
                                                overwrite: true,
                                                auto_rename: false,
                                                as_zip: previous_options.as_zip,
                                            },
                                            Some(folder),
                                        );
                                    }
                                }
                            >
                                "Overwrite"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </>
    }
}
