use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn ErrorDisplay(state: AppState) -> impl IntoView {
    view! {
        <Show when=move || state.error.get().is_some()>
            <div class="error show">
                {move || state.error.get().unwrap_or_default()}
            </div>
        </Show>
    }
}
