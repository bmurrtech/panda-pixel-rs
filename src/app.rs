use leptos::prelude::*;
use crate::components::*;
use crate::state::AppState;

#[component]
pub fn App() -> impl IntoView {
    let app_state = AppState::new();

    view! {
        <div class="container">
            <div class="header">
                <h1>
                    <img
                        src="./assets/icon_32x32.png"
                        alt="Panda Pixel Logo"
                        class="header-logo"
                    />
                    "Panda Pixel"
                </h1>
                <p>"Fast, 100% private image compression"</p>
            </div>

            <FileSelector state=app_state.clone() />
            <QualitySlider state=app_state.clone() />
            <FormatSelector state=app_state.clone() />
            <AdvancedOptions state=app_state.clone() />
            <CompressButton state=app_state.clone() />
            <ProgressBar state=app_state.clone() />
            <StatusDisplay state=app_state.clone() />
            <ErrorDisplay state=app_state.clone() />
            <ResultsList state=app_state.clone() />
        </div>
    }
}
