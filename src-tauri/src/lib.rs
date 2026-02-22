mod commands;
mod compression;
mod compression_options;

/// Runs the Tauri desktop application.
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::select_files,
            commands::handle_dropped_files,
            commands::compress_image,
            commands::compress_batch,
            commands::select_output_folder,
            commands::save_files_to_folder,
            commands::save_file,
            commands::resize_window,
            commands::open_devtools,
            commands::open_in_file_manager,
        ]);

    if let Err(err) = builder.run(tauri::generate_context!()) {
        eprintln!("error while running tauri application: {err}");
        std::process::exit(1);
    }
}