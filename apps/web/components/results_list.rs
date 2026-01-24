use leptos::prelude::*;
use crate::state::AppState;

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024;
    let sizes = ["B", "KB", "MB", "GB"];
    let i = (bytes as f64).log(k as f64).floor() as usize;
    let i = i.min(sizes.len() - 1);
    format!("{:.1} {}", bytes as f64 / (k as f64).powi(i as i32), sizes[i])
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
                                                // Extract just the filename for display (not full path)
                                                std::path::Path::new(&result.original_path)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or(&result.original_path)
                                                    .to_string()
                                            }}
                                        </strong>
                                        <p style="color: #d1d5db; font-size: 0.875rem;">
                                            {format_bytes(result.original_size)}
                                            " → "
                                            {format_bytes(result.compressed_size)}
                                            " ("
                                            {format!("{:.1}", result.savings_percent)}
                                            "% saved)"
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
