use crate::state::AppState;
use crate::tauri_helpers;
use crate::utils;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, EventTarget};

#[component]
pub fn FileSelector(state: AppState) -> impl IntoView {
    // Set up event listener for dropped files (handled in JS, dispatched here)
    let state_clone = state.clone();
    if let Some(win) = window() {
        let target: EventTarget = win.into();
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            if let Some(custom_event) = event.dyn_ref::<web_sys::CustomEvent>() {
                let detail = custom_event.detail();
                if let Ok(detail_obj) = detail.dyn_into::<js_sys::Object>() {
                    if let Ok(files_js) =
                        js_sys::Reflect::get(&detail_obj, &JsValue::from_str("files"))
                    {
                        if let Ok(files_json) = js_sys::JSON::stringify(&files_js) {
                            if let Some(files_str) = files_json.as_string() {
                                if let Ok(files) =
                                    serde_json::from_str::<Vec<crate::state::FileInfo>>(&files_str)
                                {
                                    if utils::is_dev_mode() {
                                        let sample_names: Vec<String> =
                                            files.iter().take(3).map(|f| f.name.clone()).collect();
                                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
                                            &format!(
                                                "📦 Dropped {} files (sample: {:?})",
                                                files.len(),
                                                sample_names
                                            ),
                                        ));
                                    }
                                    // Reset state for new files (same as file picker)
                                    state_clone.is_compressing.set(false);
                                    state_clone.files.set(files);
                                    state_clone.results.set(Vec::new()); // Clear results to revert UI
                                    state_clone.has_compressed.set(false);
                                    state_clone.progress.set(0.0);
                                    state_clone.error.set(None);
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if target
            .add_event_listener_with_callback("files-dropped", closure.as_ref().unchecked_ref())
            .is_err()
        {
            // Silent failure - event listener setup issue
        }
        closure.forget(); // Keep closure alive
    }
    // Helper function to reset state when new files are selected
    let reset_state_for_new_files = move |new_files: Vec<crate::state::FileInfo>| {
        state.is_compressing.set(false);
        state.files.set(new_files);
        state.results.set(Vec::new()); // Clear results to revert UI to "Compress" state
        state.has_compressed.set(false);
        state.progress.set(0.0);
        state.error.set(None);
    };

    let select_files = move || {
        spawn_local(async move {
            match tauri_helpers::invoke_tauri::<Vec<crate::state::FileInfo>>(
                "select_files",
                JsValue::NULL,
            )
            .await
            {
                Ok(files) => {
                    if utils::is_dev_mode() {
                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                            "📁 Selected {} files",
                            files.len()
                        )));
                    }
                    reset_state_for_new_files(files);
                }
                Err(e) => {
                    if !e.contains("cancelled") && !e.contains("Dialog cancelled") {
                        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!(
                            "❌ Error selecting files: {}",
                            e
                        )));
                    }
                    state.error.set(Some(e));
                }
            }
        });
    };

    view! {
        <div
            class="upload-section"
            class:disabled=move || state.is_compressing.get()
        >
            <button
                type="button"
                class="upload-button"
                on:click=move |_| {
                    if state.is_compressing.get_untracked() {
                        return;
                    }
                    select_files();
                }
                disabled=move || state.is_compressing.get()
            >
                "📁 Select Images"
            </button>
            <p style="color: #d1d5db; font-size: 0.875rem;">
                "PNG, JPEG, BMP, TIFF, WebP, ICO supported"
            </p>
        </div>
    }
}
