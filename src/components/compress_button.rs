use leptos::prelude::*;
use crate::state::{AppState, CompressionResult, PendingSaveOptions};
use crate::backend::{BackendProvider, AppBackend, CompressionRequest, SaveFilesRequest, SaveZipRequest, CollisionCheckRequest, ResolveUniqueFilenamesRequest, FileSaveData, BackendError};
use crate::utils;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys;

/// Sanitize a filename by removing path separators and dangerous characters
fn sanitize_filename(name: &str) -> String {
    name.replace('\\', "_")
        .replace('/', "_")
        .replace(':', "_")
        .replace('*', "_")
        .replace('?', "_")
        .replace('"', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace('|', "_")
        .trim()
        .to_string()
}

/// Check if a filename edit is "dirty" (different from initial value after sanitization)
fn is_dirty(edit: &str, initial: &str) -> bool {
    sanitize_filename(edit) != sanitize_filename(initial)
}

/// Build the final output filenames considering collision overrides
fn build_output_filenames(
    results: &[CompressionResult],
    collision_files: &[String],
    collision_edits: &[String],
    as_zip: bool,
    base_zip_filename: Option<&str>,
) -> Vec<String> {
    if as_zip {
        if collision_files.len() == 1 && collision_edits.len() == 1 {
            vec![sanitize_filename(&collision_edits[0])]
        } else if let Some(zip_name) = base_zip_filename {
            vec![zip_name.to_string()]
        } else {
            vec![build_zip_filename()]
        }
    } else {
        results
            .iter()
            .map(|result| {
                let default_name = generate_filename_from_result(result);
                if let Some(index) = collision_files.iter().position(|c| c == &default_name) {
                    collision_edits
                        .get(index)
                        .map(|edit| sanitize_filename(edit))
                        .unwrap_or_else(|| default_name)
                } else {
                    default_name
                }
            })
            .collect()
    }
}

/// Build one proposed filename per compression result for collision "Rename" save (non-ZIP).
/// Collision rows match results by export default name in order; dirty rows use the edited name,
/// clean rows keep the default so disk resolution can assign ` (1)`, `(2)`, …
fn build_proposed_filenames_for_collision_rename(
    results: &[CompressionResult],
    collision_files: &[String],
    collision_edits: &[String],
    collision_initial: &[String],
) -> Vec<String> {
    let mut row_used = vec![false; collision_files.len()];
    let mut out = Vec::with_capacity(results.len());
    for result in results {
        let default_name = generate_filename_from_result(result);
        if let Some(i) = (0..collision_files.len()).find(|&idx| {
            !row_used[idx] && collision_files[idx] == default_name
        }) {
            row_used[i] = true;
            let dirty = collision_edits
                .get(i)
                .zip(collision_initial.get(i))
                .map(|(e, init)| is_dirty(e, init))
                .unwrap_or(false);
            if dirty {
                let edit = collision_edits
                    .get(i)
                    .map(|s| sanitize_filename(s))
                    .unwrap_or_else(|| default_name.clone());
                out.push(edit);
            } else {
                out.push(default_name);
            }
        } else {
            out.push(default_name);
        }
    }
    out
}

/// Build files payload with optional filename overrides
fn prepare_files_payload_with_overrides(
    results: &[CompressionResult],
    filenames: Vec<String>,
) -> Vec<FileSaveData> {
    results
        .iter()
        .zip(filenames.into_iter())
        .map(|(result, filename)| {
            FileSaveData {
                filename,
                data: result.data.clone(),
            }
        })
        .collect()
}

fn generate_filename_from_result(result: &CompressionResult) -> String {
    result.display_export_filename()
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
) -> Vec<FileSaveData> {
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

        files_to_save.push(FileSaveData {
            filename,
            data: result.data.clone(),
        });
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

async fn save_with_options(
    state: AppState,
    backend: BackendProvider,
    options: PendingSaveOptions,
    folder_override: Option<String>,
    manual_filenames: Option<Vec<String>>,
) {
    let results = state.results.get_untracked();

    if results.is_empty() {
        state.error.set(Some("No compressed files to save".to_string()));
        state.status.set(None);
        return;
    }

    let is_tauri_backend = backend.is_tauri();
    
    let output_folder = match folder_override {
        Some(folder) => folder,
        None => match backend.select_output_folder().await {
            Ok(folder) => folder,
            Err(BackendError::Cancelled) => return,
            Err(e) => {
                state.status.set(None);
                state.error.set(Some(e.to_string()));
                return;
            }
        },
    };

    let base_zip_filename = if options.as_zip {
        Some(build_zip_filename())
    } else {
        None
    };

    let collisions = if is_tauri_backend {
        let collision_targets: Vec<String> = if let Some(ref zip_filename) = base_zip_filename {
            vec![zip_filename.clone()]
        } else {
            results.iter().map(generate_filename_from_result).collect()
        };

        match backend.check_file_collisions(CollisionCheckRequest {
            output_folder: output_folder.clone(),
            filenames: collision_targets,
        }).await {
            Ok(collisions) => {
                if !collisions.is_empty() && !options.overwrite && !options.auto_rename && manual_filenames.is_none() {
                    let pending_opts = PendingSaveOptions {
                        overwrite: options.overwrite,
                        auto_rename: options.auto_rename,
                        as_zip: options.as_zip,
                    };
                    state.init_collision_state(collisions, output_folder, pending_opts);
                    return;
                }
                collisions
            }
            Err(e) => {
                state.status.set(None);
                state.error.set(Some(e.to_string()));
                return;
            }
        }
    } else {
        Vec::new()
    };

    state.reset_collision_state();

    let files_to_save = if let Some(filenames) = manual_filenames {
        prepare_files_payload_with_overrides(&results, filenames)
    } else if is_tauri_backend && options.auto_rename {
        let default_names: Vec<String> = results.iter().map(generate_filename_from_result).collect();
        match backend
            .resolve_unique_filenames(ResolveUniqueFilenamesRequest {
                output_folder: output_folder.clone(),
                filenames: default_names,
            })
            .await
        {
            Ok(resolved) => prepare_files_payload_with_overrides(&results, resolved),
            Err(e) => {
                state.status.set(None);
                state.error.set(Some(e.to_string()));
                return;
            }
        }
    } else {
        prepare_files_payload(&results, &output_folder, options.auto_rename, &collisions)
    };

    if options.as_zip {
        let zip_filename = base_zip_filename.unwrap_or_else(build_zip_filename);
        let final_zip_filename = if is_tauri_backend && options.auto_rename {
            match backend
                .resolve_unique_filenames(ResolveUniqueFilenamesRequest {
                    output_folder: output_folder.clone(),
                    filenames: vec![zip_filename.clone()],
                })
                .await
            {
                Ok(mut v) => v.pop().unwrap_or(zip_filename),
                Err(e) => {
                    state.status.set(None);
                    state.error.set(Some(e.to_string()));
                    return;
                }
            }
        } else {
            zip_filename
        };

        match backend.save_files_as_zip(SaveZipRequest {
            output_folder: output_folder.clone(),
            zip_filename: final_zip_filename,
            files: files_to_save,
        }).await {
            Ok(saved_zip_path) => {
                state.error.set(None);
                if is_tauri_backend {
                    state.status.set(Some(format!("Saved ZIP: {}", utils::basename(&saved_zip_path))));
                } else {
                    utils::product_log(&format!(
                        "💾 ZIP download: {} ({} file(s) bundled)",
                        utils::basename(&saved_zip_path),
                        results.len()
                    ));
                    state.status.set(Some(format!(
                        "Downloaded ZIP: {}",
                        utils::basename(&saved_zip_path)
                    )));
                }
                let _ = backend.open_in_file_manager(output_folder.clone()).await;
            }
            Err(e) => {
                state.status.set(None);
                state.error.set(Some(e.to_string()));
            }
        }

        return;
    }

    match backend.save_files_to_folder(SaveFilesRequest {
        output_folder: output_folder.clone(),
        files: files_to_save,
    }).await {
        Ok(saved_paths) => {
            state.error.set(None);
            state.status.set(Some(format!(
                "Saved {} file(s) to {}",
                saved_paths.len(),
                utils::basename(&output_folder)
            )));
            let _ = backend.open_in_file_manager(output_folder.clone()).await;
        }
        Err(e) => {
            state.status.set(None);
            state.error.set(Some(e.to_string()));
        }
    }
}

fn spawn_save_with_options(
    state: AppState,
    backend: BackendProvider,
    options: PendingSaveOptions,
    folder_override: Option<String>,
    manual_filenames: Option<Vec<String>>,
) {
    spawn_local(async move {
        save_with_options(state, backend, options, folder_override, manual_filenames).await;
    });
}

#[component]
pub fn CompressButton(state: AppState) -> impl IntoView {
    let backend = BackendProvider::new();
    let is_tauri = backend.is_tauri();

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
    let has_single_result = move || state.results.get().len() == 1;
    let save_label = move || {
        if has_multiple_results() {
            "Save Files"
        } else {
            "Save File"
        }
    };

    let is_dirty = move || {
        let edits = state.collision_name_edits.get();
        let initial = state.collision_initial_snapshot.get();
        if edits.len() != initial.len() {
            return false;
        }
        edits.iter().zip(initial.iter()).any(|(edit, init)| is_dirty(edit, init))
    };

    let has_empty_name = move || {
        state.collision_name_edits.get().iter().any(|name| sanitize_filename(name).is_empty())
    };

    let compress_files = move |_| {
        spawn_local(async move {
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::sync::Arc;

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

            state.progress.set(25.0);
            utils::sleep_ms(50).await;

            let done = Arc::new(AtomicBool::new(false));
            let done_ticker = done.clone();
            let state_ticker = state.clone();
            spawn_local(async move {
                for step in [50.0_f64, 75.0_f64] {
                    utils::sleep_ms(450).await;
                    if done_ticker.load(Ordering::SeqCst) {
                        break;
                    }
                    if state_ticker.is_compressing.get_untracked() {
                        state_ticker.progress.set(step);
                    }
                }
            });

            let batch_result = BackendProvider::new()
                .compress_batch(CompressionRequest {
                    file_paths,
                    compression_level,
                    output_format: output_format.clone(),
                    oxipng,
                    png_lossy,
                })
                .await;

            done.store(true, Ordering::SeqCst);

            match batch_result {
                Ok(results) => {
                    let n = results.len();
                    let snap = Some(output_format);
                    let results: Vec<_> = results
                        .into_iter()
                        .map(|mut r| {
                            if r.requested_output_format.is_none() {
                                r.requested_output_format = snap.clone();
                            }
                            r
                        })
                        .collect();
                    state.results.set(results);
                    state.has_compressed.set(true);
                    state.progress.set(100.0);
                    utils::product_log(&format!(
                        "✅ Compression finished: {} result(s) ready to save or download",
                        n
                    ));
                    utils::sleep_ms(220).await;
                }
                Err(e) => {
                    state.progress.set(0.0);
                    state.error.set(Some(e.to_string()));
                }
            }

            state.is_compressing.set(false);
            state.progress.set(0.0);
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
                            <Show
                                when=move || is_tauri
                                fallback=move || view! {
                                    <Show
                                        when=has_single_result
                                        fallback=move || view! {
                                            <button
                                                type="button"
                                                class="convert-button show save-zip-button"
                                                disabled=move || state.is_compressing.get()
                                                on:click=move |_| {
                                                    spawn_save_with_options(
                                                        state.clone(),
                                                        BackendProvider::new(),
                                                        PendingSaveOptions {
                                                            overwrite: false,
                                                            auto_rename: false,
                                                            as_zip: true,
                                                        },
                                                        None,
                                                        None,
                                                    );
                                                }
                                            >
                                                "Save All (.zip)"
                                            </button>
                                        }
                                    >
                                        <div class="save-actions save-actions-web">
                                            <button
                                                type="button"
                                                class="convert-button show save-primary-button"
                                                disabled=move || state.is_compressing.get()
                                                on:click=move |_| {
                                                    spawn_save_with_options(
                                                        state.clone(),
                                                        BackendProvider::new(),
                                                        PendingSaveOptions {
                                                            overwrite: false,
                                                            auto_rename: false,
                                                            as_zip: false,
                                                        },
                                                        None,
                                                        None,
                                                    );
                                                }
                                            >
                                                "Save File"
                                            </button>
                                            <button
                                                type="button"
                                                class="convert-button show save-zip-button"
                                                disabled=move || state.is_compressing.get()
                                                on:click=move |_| {
                                                    spawn_save_with_options(
                                                        state.clone(),
                                                        BackendProvider::new(),
                                                        PendingSaveOptions {
                                                            overwrite: false,
                                                            auto_rename: false,
                                                            as_zip: true,
                                                        },
                                                        None,
                                                        None,
                                                    );
                                                }
                                            >
                                                "Save as .zip"
                                            </button>
                                        </div>
                                    </Show>
                                }
                            >
                                // Desktop mode: show both Save Files and Save All (.zip)
                                <button
                                    type="button"
                                    class="convert-button show save-primary-button"
                                    disabled=move || state.is_compressing.get()
                                    on:click=move |_| {
                                        spawn_save_with_options(
                                            state.clone(),
                                            BackendProvider::new(),
                                            PendingSaveOptions {
                                                overwrite: false,
                                                auto_rename: false,
                                                as_zip: false,
                                            },
                                            None,
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
                                                BackendProvider::new(),
                                                PendingSaveOptions {
                                                    overwrite: false,
                                                    auto_rename: false,
                                                    as_zip: true,
                                                },
                                                None,
                                                None,
                                            );
                                        }
                                    >
                                        "Save All (.zip)"
                                    </button>
                                </Show>
                            </Show>
                        </div>
                    </Show>
                </Show>
            </div>

            <Show when=move || state.show_collision_modal.get()>
                <div class="modal-overlay">
                    <div class="modal-content">
                        <h2>"File Collision Detected"</h2>
                        <p>{move || {
                            if state.collision_files.get().len() > 1 {
                                "Tip: Edit file names"
                            } else {
                                "Tip: Edit file name"
                            }
                        }}</p>
                        <ul class="collision-list">
                            <For
                                each=move || state.collision_files.get().into_iter().enumerate()
                                key=|(index, _)| *index
                                children=move |(index, filename)| {
                                    let state_for_input = state.clone();
                                    view! {
                                        <li class="collision-item">
                                            <input
                                                type="text"
                                                class="collision-input"
                                                prop:value=move || {
                                                    state_for_input.collision_name_edits.get()
                                                        .get(index)
                                                        .cloned()
                                                        .unwrap_or_default()
                                                }
                                                on:input=move |ev| {
                                                    let target: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
                                                    let value = target.value();
                                                    let mut edits = state_for_input.collision_name_edits.get_untracked();
                                                    if index < edits.len() {
                                                        edits[index] = value;
                                                        state_for_input.collision_name_edits.set(edits);
                                                    }
                                                }
                                            />
                                            <span class="original-name">{filename}</span>
                                        </li>
                                    }
                                }
                            />
                        </ul>
                        <div class="modal-actions">
                            <button
                                type="button"
                                class="btn-secondary"
                                on:click=move |_| {
                                    state.reset_collision_state();
                                }
                            >
                                "Cancel"
                            </button>
                            <Show when=move || !is_dirty()>
                                <button
                                    type="button"
                                    class="btn-primary"
                                    on:click=move |_| {
                                    if let (Some(folder), Some(previous_options)) = (
                                        state.pending_save_folder.get_untracked(),
                                        state.pending_save_options.get_untracked(),
                                    ) {
                                        spawn_save_with_options(
                                            state.clone(),
                                            BackendProvider::new(),
                                            PendingSaveOptions {
                                                overwrite: false,
                                                auto_rename: true,
                                                as_zip: previous_options.as_zip,
                                            },
                                            Some(folder),
                                            None,
                                        );
                                    }
                                }
                            >
                                "Duplicate"
                            </button>
                            </Show>
                            <Show
                                when=is_dirty
                                fallback=move || view! {
                                    <button
                                        type="button"
                                        class="btn-danger"
                                        on:click=move |_| {
                                            if let (Some(folder), Some(previous_options)) = (
                                                state.pending_save_folder.get_untracked(),
                                                state.pending_save_options.get_untracked(),
                                            ) {
                                                spawn_save_with_options(
                                                    state.clone(),
                                                    BackendProvider::new(),
                                                    PendingSaveOptions {
                                                        overwrite: true,
                                                        auto_rename: false,
                                                        as_zip: previous_options.as_zip,
                                                    },
                                                    Some(folder),
                                                    None,
                                                );
                                            }
                                        }
                                    >
                                        "Overwrite"
                                    </button>
                                }
                            >
                                <button
                                    type="button"
                                    class="btn-rename-safe"
                                    disabled=has_empty_name
                                    on:click=move |_| {
                                        let state = state.clone();
                                        spawn_local(async move {
                                            let Some(folder) = state.pending_save_folder.get_untracked() else { return };
                                            let Some(previous_options) = state.pending_save_options.get_untracked() else { return };
                                            let results = state.results.get_untracked();

                                            let collision_files = state.collision_files.get_untracked();
                                            let collision_edits = state.collision_name_edits.get_untracked();
                                            let collision_initial = state.collision_initial_snapshot.get_untracked();

                                            if collision_edits.iter().any(|name| sanitize_filename(name).is_empty()) {
                                                state.error.set(Some("File name cannot be empty".to_string()));
                                                return;
                                            }

                                            let backend = BackendProvider::new();

                                            if previous_options.as_zip {
                                                let base_zip = Some(build_zip_filename());
                                                let output_names = build_output_filenames(
                                                    &results,
                                                    &collision_files,
                                                    &collision_edits,
                                                    true,
                                                    base_zip.as_deref(),
                                                );

                                                match backend.check_file_collisions(CollisionCheckRequest {
                                                    output_folder: folder.clone(),
                                                    filenames: output_names.clone(),
                                                }).await {
                                                    Ok(new_collisions) => {
                                                        if !new_collisions.is_empty() {
                                                            state.error.set(Some(
                                                                "That file already exists. Choose a different name, or Cancel.".to_string(),
                                                            ));
                                                        } else {
                                                            state.reset_collision_state();
                                                            spawn_save_with_options(
                                                                state.clone(),
                                                                BackendProvider::new(),
                                                                PendingSaveOptions {
                                                                    overwrite: false,
                                                                    auto_rename: false,
                                                                    as_zip: true,
                                                                },
                                                                Some(folder),
                                                                Some(output_names),
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        state.error.set(Some(e.to_string()));
                                                    }
                                                }
                                                return;
                                            }

                                            let proposed = build_proposed_filenames_for_collision_rename(
                                                &results,
                                                &collision_files,
                                                &collision_edits,
                                                &collision_initial,
                                            );

                                            let resolved = match backend.resolve_unique_filenames(ResolveUniqueFilenamesRequest {
                                                output_folder: folder.clone(),
                                                filenames: proposed,
                                            }).await {
                                                Ok(v) => v,
                                                Err(e) => {
                                                    state.error.set(Some(e.to_string()));
                                                    return;
                                                }
                                            };

                                            match backend.check_file_collisions(CollisionCheckRequest {
                                                output_folder: folder.clone(),
                                                filenames: resolved.clone(),
                                            }).await {
                                                Ok(new_collisions) => {
                                                    if !new_collisions.is_empty() {
                                                        state.error.set(Some(
                                                            "That file name is not available. Choose a different name, or Cancel.".to_string(),
                                                        ));
                                                    } else {
                                                        state.reset_collision_state();
                                                        spawn_save_with_options(
                                                            state.clone(),
                                                            BackendProvider::new(),
                                                            PendingSaveOptions {
                                                                overwrite: false,
                                                                auto_rename: false,
                                                                as_zip: false,
                                                            },
                                                            Some(folder),
                                                            Some(resolved),
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    state.error.set(Some(e.to_string()));
                                                }
                                            }
                                        });
                                    }
                                >
                                    "Rename"
                                </button>
                            </Show>
                        </div>
                    </div>
                </div>
            </Show>
        </>
    }
}