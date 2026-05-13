mod app;
mod backend;
mod components;
mod state;
mod tauri_helpers;
mod utils;

use leptos::mount::mount_to_body;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    crate::utils::product_log("🚀 Panda Pixel starting (browser mode)");
    if cfg!(debug_assertions) {
        crate::utils::product_log("Leptos app starting (debug build)");
    }

    mount_to_body(|| leptos::view! { <app::App /> });
}
