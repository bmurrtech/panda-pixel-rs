use leptos::prelude::*;
use crate::state::AppState;

#[component]
pub fn AdvancedOptions(state: AppState) -> impl IntoView {
    view! {
        <div>
            <div
                class="advanced-toggle"
                on:click=move |_| {
                    state.advanced_open.update(|open| *open = !*open);
                }
            >
                <span>"Advanced Options"</span>
                <span class="advanced-arrow" class:open=move || state.advanced_open.get()>
                    "▼"
                </span>
            </div>
            <Show when=move || state.advanced_open.get()>
                <div class="advanced-options" class:show=move || state.advanced_open.get()>
                    <div class="checkbox-group">
                        <input
                            type="checkbox"
                            id="oxipng"
                            checked=move || state.oxipng.get()
                            on:change=move |ev| {
                                state.oxipng.set(event_target_checked(&ev));
                            }
                        />
                        <label for="oxipng">"Enable oxipng optimization"</label>
                    </div>
                    <div class="checkbox-group">
                        <input
                            type="checkbox"
                            id="pngLossy"
                            checked=move || state.png_lossy.get()
                            on:change=move |ev| {
                                state.png_lossy.set(event_target_checked(&ev));
                            }
                        />
                        <label for="pngLossy">"PNG lossy compression"</label>
                    </div>
                </div>
            </Show>
        </div>
    }
}
