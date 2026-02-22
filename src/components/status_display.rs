use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn StatusDisplay(state: AppState) -> impl IntoView {
    view! {
        <Show when=move || state.status.get().is_some()>
            <div class="status show">
                {move || state.status.get().unwrap_or_default()}
            </div>
        </Show>
    }
}
