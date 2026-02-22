use leptos::prelude::*;
use crate::state::AppState;
use std::path::Path;

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024;
    let sizes = vec!["B", "KB", "MB", "GB"];
    let i = (bytes as f64).log(k as f64).floor() as usize;
    let i = i.min(sizes.len() - 1);
    format!("{:.1} {}", bytes as f64 / (k as f64).powi(i as i32), sizes[i])
}

fn ext_from_mime(mime: &str) -> Option<&'static str> {
    match mime {
        "image/webp" => Some("webp"),
        "image/avif" => Some("avif"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/tiff" => Some("tiff"),
        "image/bmp" => Some("bmp"),
        "image/x-icon" => Some("ico"),
        _ => None,
    }
}

fn display_output_filename(original_path: &str, mime_type: &str) -> String {
    let path = Path::new(original_path);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("compressed");
    let fallback_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let ext = ext_from_mime(mime_type).unwrap_or(fallback_ext);
    format!("{stem}.{ext}")
}

#[component]
pub fn ResultsList(state: AppState) -> impl IntoView {
    view! {
        <Show when=move || !state.results.get().is_empty()>
            <div class="results show">
                <h3 style="margin-bottom: 1rem; color: #ffffff;">"Compression Results"</h3>
                <For
                    each=move || state.results.get()
                    key=|result| result.original_path.clone()
                    children=move |result| {
                        view! {
                            <div class="result-item">
                                <div class="result-stats">
                                    <div>
                                        <strong>
                                            {move || {
                                                display_output_filename(&result.original_path, &result.mime_type)
                                            }}
                                        </strong>
                                        <p style="color: #d1d5db; font-size: 0.875rem;">
                                            {format_bytes(result.original_size)}
                                            " → "
                                            {format_bytes(result.compressed_size)}
                                            " ("
                                            {format!("{:+.1}%", result.savings_percent)}
                                            ")"
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </Show>
    }
}
