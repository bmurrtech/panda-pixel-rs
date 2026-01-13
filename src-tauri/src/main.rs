// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod compression;
mod compression_options;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            select_files,
            handle_dropped_files,
            compress_image,
            compress_batch,
            select_output_folder,
            save_files_to_folder,
            save_file,
            resize_window,
            open_devtools
        ])
        .setup(|app| {
            // Enable global Tauri API on window object
            // This ensures window.__TAURI__ is available
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(_window) = app.get_webview_window("main") {
                    log::info!("Webview window found, Tauri API should be available");
                    // Open devtools automatically in debug mode
                    // Note: Devtools will open when the window is ready
                    std::thread::spawn({
                        let app_handle = app.handle().clone();
                        move || {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            if let Some(webview) = app_handle.get_webview_window("main") {
                                webview.open_devtools();
                            }
                        }
                    });
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
