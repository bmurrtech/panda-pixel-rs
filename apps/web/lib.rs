mod app;
mod components;
mod state;
mod tauri_helpers;
mod utils;

use wasm_bindgen::prelude::*;
use leptos::mount::mount_to_body;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    // Minimal startup logging - only in dev mode
    if cfg!(debug_assertions) {
        web_sys::console::log_1(&JsValue::from_str("Leptos app starting"));
    }
    
    mount_to_body(|| leptos::view! { <app::App /> });
}
