use leptos::prelude::*;
use crate::state::AppState;
use crate::backend::{BackendProvider, AppBackend, BackendError};
use crate::utils;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, EventTarget, HtmlInputElement};

#[component]
pub fn FileSelector(state: AppState) -> impl IntoView {
    let file_input_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let backend = BackendProvider::new();
    let backend_for_select = backend.clone();

    let has_native_dialogs = backend.capabilities().supports_native_dialogs;

    let reset_state_for_new_files = move |new_files: Vec<crate::state::FileInfo>| {
        state.is_compressing.set(false);
        state.files.set(new_files);
        state.results.set(Vec::new());
        state.has_compressed.set(false);
        state.progress.set(0.0);
        state.error.set(None);
        state.status.set(None);
    };

    let reset_for_drop = reset_state_for_new_files.clone();
    if let Some(win) = window() {
        let target: EventTarget = win.into();
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            if let Some(custom_event) = event.dyn_ref::<web_sys::CustomEvent>() {
                let detail = custom_event.detail();
                if let Ok(detail_obj) = detail.dyn_into::<js_sys::Object>() {
                    if let Ok(files_js) = js_sys::Reflect::get(&detail_obj, &JsValue::from_str("files")) {
                        if let Ok(files_json) = js_sys::JSON::stringify(&files_js) {
                            if let Some(files_str) = files_json.as_string() {
                                if let Ok(files) = serde_json::from_str::<Vec<crate::state::FileInfo>>(&files_str) {
                                    utils::product_log(&format!(
                                        "📦 Dropped {} file(s)",
                                        files.len()
                                    ));
                                    reset_for_drop(files);
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Err(_) = target.add_event_listener_with_callback("files-dropped", closure.as_ref().unchecked_ref()) {
            // Silent failure - event listener setup issue
        }
        closure.forget();
    }

    let handle_browser_files = move |file_list: web_sys::FileList| {
        let mut files = Vec::new();
        let mut file_objects = js_sys::Array::new();
        let length = file_list.length();
        
        for i in 0..length {
            let file = js_sys::Reflect::get(&file_list, &JsValue::from_f64(i as f64)).ok();
            if let Some(file_val) = file {
                if let Ok(file) = file_val.dyn_into::<web_sys::File>() {
                    let name = file.name();
                    let size = file.size() as u64;
                    let path = format!("browser://{}", name);
                    files.push(crate::state::FileInfo { path: path.clone(), name: name.clone(), size });
                    
                    // Store file object for later compression
                    let file_obj = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(&file_obj, &JsValue::from_str("path"), &JsValue::from_str(&path));
                    let _ = js_sys::Reflect::set(&file_obj, &JsValue::from_str("name"), &JsValue::from_str(&name));
                    let _ = js_sys::Reflect::set(&file_obj, &JsValue::from_str("size"), &JsValue::from_f64(size as f64));
                    let _ = js_sys::Reflect::set(&file_obj, &JsValue::from_str("file"), &file);
                    file_objects.push(&file_obj);
                }
            }
        }
        
        // Store files globally for compression
        if let Some(win) = window() {
            let _ = js_sys::Reflect::set(&win, &JsValue::from_str("__BROWSER_DROPPED_FILES"), &file_objects);
        }

        utils::product_log(&format!(
            "📁 Browser selected {} file(s) (stored for compression)",
            files.len()
        ));

        reset_state_for_new_files(files);
    };

    let select_files_backend = move || {
        let state = state.clone();
        let backend = backend_for_select.clone();
        spawn_local(async move {
            match backend.select_files().await {
                Ok(files) => {
                    if utils::is_dev_mode() {
                        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("📁 Selected {} files", files.len())));
                    }
                    state.is_compressing.set(false);
                    state.files.set(files);
                    state.results.set(Vec::new());
                    state.has_compressed.set(false);
                    state.progress.set(0.0);
                    state.error.set(None);
                    state.status.set(None);
                }
                Err(BackendError::Cancelled) => {
                    // User cancelled, no error
                }
                Err(BackendError::NotAvailable) => {
                    // Backend doesn't support file selection, trigger browser input
                    if let Some(input) = file_input_ref.get() {
                        input.click();
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    if !msg.contains("cancelled") && !msg.contains("Dialog cancelled") {
                        web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!("❌ Error selecting files: {}", msg)));
                        state.error.set(Some(msg));
                    }
                }
            }
        });
    };

    let select_files_browser = move || {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    view! {
        <div
            class="upload-section"
            class:disabled=move || state.is_compressing.get()
        >
            <input
                type="file"
                multiple=true
                accept="image/png,image/jpeg,image/jpg,image/bmp,image/tiff,image/webp,image/ico"
                style="display: none;"
                node_ref=file_input_ref
                on:change=move |ev| {
                    let target = ev.target().unwrap();
                    let input: HtmlInputElement = target.dyn_into().unwrap();
                    if let Some(files) = input.files() {
                        handle_browser_files(files);
                    }
                    input.set_value("");
                }
            />
            <button
                type="button"
                class="upload-button"
                on:click=move |_| {
                    if state.is_compressing.get_untracked() {
                        return;
                    }
                    if has_native_dialogs {
                        select_files_backend();
                    } else {
                        select_files_browser();
                    }
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