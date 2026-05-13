use crate::state::{AppState, CompressionResult};
use crate::utils;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let k = 1024;
    let sizes = vec!["B", "KB", "MB", "GB"];
    let i = (bytes as f64).log(k as f64).floor() as usize;
    let i = i.min(sizes.len() - 1);
    format!(
        "{:.1} {}",
        bytes as f64 / (k as f64).powi(i as i32),
        sizes[i]
    )
}

fn download_single_file(result: &CompressionResult) {
    let display_name = result.display_export_filename();

    let window = web_sys::window().expect("no window");
    let document = window.document().expect("no document");

    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &js_sys::Array::of1(&js_sys::Uint8Array::from(&result.data[..])),
        web_sys::BlobPropertyBag::new().type_(&result.mime_type),
    )
    .expect("Failed to create blob");

    let url = web_sys::Url::create_object_url_with_blob(&blob).expect("Failed to create URL");

    let anchor = document
        .create_element("a")
        .expect("Failed to create anchor")
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .expect("Failed to cast anchor");

    anchor.set_href(&url);
    anchor.set_download(&display_name);
    let html_el: &web_sys::HtmlElement = anchor.unchecked_ref();
    let _ = html_el.style().set_property("display", "none");

    document
        .body()
        .expect("no body")
        .append_child(&anchor)
        .expect("Failed to append anchor");

    anchor.click();

    document
        .body()
        .unwrap()
        .remove_child(&anchor)
        .expect("Failed to remove anchor");

    let _ = web_sys::Url::revoke_object_url(&url);

    utils::product_log(&format!(
        "⬇️ Started download: {} ({} bytes)",
        display_name,
        result.data.len()
    ));
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
                        let row = result.clone();
                        let result_for_click = result.clone();
                        view! {
                            <div
                                class="result-item"
                                on:click=move |_| {
                                    download_single_file(&result_for_click);
                                }
                                style="cursor: pointer;"
                            >
                                <div class="result-stats">
                                    <div>
                                        <strong>{row.display_export_filename()}</strong>
                                        <p style="color: #d1d5db; font-size: 0.875rem;">
                                            {format_bytes(row.original_size)}
                                            " → "
                                            {format_bytes(row.compressed_size)}
                                            " ("
                                            {format!("{:+.1}%", row.savings_percent)}
                                            ")"
                                        </p>
                                    </div>
                                </div>
                                <div class="result-actions" style="padding-left: 0.5rem;">
                                    <svg
                                        width="22"
                                        height="22"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="#10b981"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        aria-hidden="true"
                                    >
                                        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                                        <polyline points="7 10 12 15 17 10"></polyline>
                                        <line x1="12" y1="15" x2="12" y2="3"></line>
                                    </svg>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </Show>
    }
}
