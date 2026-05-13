// Utility functions for logging and path handling

use std::path::Path;

/// Benign, user-visible-ish console breadcrumbs (safe for release builds; no paths to secrets).
#[cfg(target_arch = "wasm32")]
pub fn product_log(message: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(message));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn product_log(_message: &str) {}

/// Wait for UI timers (WASM). Used for staged progress during long async work.
#[cfg(target_arch = "wasm32")]
pub async fn sleep_ms(ms: u32) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    if ms == 0 {
        return;
    }
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let Some(win) = web_sys::window() else {
            let f: &js_sys::Function = resolve.unchecked_ref();
            let _ = f.call0(&JsValue::UNDEFINED);
            return;
        };
        let f: &js_sys::Function = resolve.unchecked_ref();
        if win
            .set_timeout_with_callback_and_timeout_and_arguments_0(f, ms as i32)
            .is_err()
        {
            let _ = f.call0(&JsValue::UNDEFINED);
        }
    });
    let _ = JsFuture::from(promise).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep_ms(_ms: u32) {}

/// Extract just the filename from a path
pub fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Extract just the directory name from a path (for logging)
pub fn dirname(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or(".")
        .to_string()
}

/// Check if we're in dev mode (for conditional logging)
pub fn is_dev_mode() -> bool {
    // In release builds, this will be false
    cfg!(debug_assertions)
}
