use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn CollisionModal(state: AppState) -> impl IntoView {
    let handle_cancel = move |_| {
        state.show_collision_modal.set(false);
        state.pending_save_folder.set(None);
        state.collision_files.set(Vec::new());
    };

    let handle_rename = move |_| {
        state.show_collision_modal.set(false);
    };

    let handle_overwrite = move |_| {
        state.show_collision_modal.set(false);
    };

    view! {
        <Show when=move || state.show_collision_modal.get()>
            <div class="modal-overlay">
                <div class="modal-content">
                    <h2>"File Collision Detected"</h2>
                    <p>"The following files already exist in the selected folder:"</p>
                    <ul class="collision-list">
                        <For
                            each=move || state.collision_files.get()
                            key=|filename| filename.clone()
                            children=move |filename| {
                                view! { <li>{filename}</li> }
                            }
                        />
                    </ul>
                    <p>"Would you like to overwrite them or automatically rename?"</p>
                    <div class="modal-actions">
                        <button type="button" class="btn-secondary" on:click=handle_cancel>
                            "Cancel"
                        </button>
                        <button type="button" class="btn-primary" on:click=handle_rename>
                            "Rename"
                        </button>
                        <button type="button" class="btn-danger" on:click=handle_overwrite>
                            "Overwrite"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
