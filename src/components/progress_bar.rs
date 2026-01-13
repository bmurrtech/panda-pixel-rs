use leptos::prelude::*;
use crate::state::AppState;

#[component]
pub fn ProgressBar(state: AppState) -> impl IntoView {
    let show = move || state.is_compressing.get();
    let progress = move || state.progress.get();

    view! {
        <Show when=show>
            <div class="progress show">
                <div
                    class="progress-bar"
                    style=move || format!("width: {}%", progress())
                ></div>
            </div>
        </Show>
    }
}
