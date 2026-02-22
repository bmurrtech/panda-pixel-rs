use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn FormatSelector(state: AppState) -> impl IntoView {
    let formats = vec![
        ("original", "Auto", "#6b7280"),
        ("png", "PNG", "#5C6BC0"),
        ("jpeg", "JPEG", "#66BB6A"),
        ("webp", "WebP", "#AB47BC"),
        ("avif", "AVIF", "#F06292"),
        ("tiff", "TIFF", "#F44336"),
        ("bmp", "BMP", "#BDBDBD"),
        ("ico", "ICO", "#009688"),
    ];

    view! {
        <div class="option-group">
            <label>"Output Format"</label>
            <div class="format-buttons">
                {formats.into_iter().map(|(format, label, color)| {
                    let format_str = format.to_string();
                    let format_str_for_active = format_str.clone();
                    let format_str_for_click = format_str.clone();
                    let format_str_for_click_cloned = format_str_for_click.clone();
                    let is_active = move || state.output_format.get() == format_str_for_active;
                    view! {
                        <button
                            type="button"
                            class="format-btn"
                            class:active=is_active
                            style:background-color=color
                            on:click=move |_| {
                                state.output_format.set(format_str_for_click_cloned.clone());
                            }
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
