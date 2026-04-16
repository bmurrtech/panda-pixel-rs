use crate::state::AppState;
use leptos::prelude::*;

fn format_file_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    }
}

#[component]
pub fn FilesList(state: AppState) -> impl IntoView {
    view! {
        <Show when=move || state.show_files_list.get() && !state.files.get().is_empty()>
            <div class="results show">
                <h3 style="margin-bottom: 1rem; color: #ffffff;">
                    {format!("Selected files ({}):", state.files.get().len())}
                </h3>
                <For
                    each=move || state.files.get()
                    key=|file| file.path.clone()
                    children=move |file| {
                        view! {
                            <div class="result-item">
                                <div class="result-stats">
                                    <div>
                                        <strong style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                                            {file.name.clone()}
                                        </strong>
                                        <p style="color: #d1d5db; font-size: 0.875rem;">
                                            {format_file_size(file.size)}
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
