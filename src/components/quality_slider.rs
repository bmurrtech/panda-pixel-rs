use leptos::prelude::*;
use crate::state::AppState;

#[component]
pub fn QualitySlider(state: AppState) -> impl IntoView {
    let quality_values = vec!["low", "mid", "max"];

    let slider_value = move || {
        match state.compression_level.get().as_str() {
            "low" => 1,
            "mid" => 2,
            "max" => 3,
            _ => 2,
        }
    };

    let quality_label = move || {
        match state.compression_level.get().as_str() {
            "low" => "Low (Best Quality)",
            "mid" => "Mid (Recommended)",
            "max" => "Max (Smallest File)",
            _ => "Mid (Recommended)",
        }
    };

    view! {
        <div class="option-group">
            <label>"Compression Level"</label>
            <div class="quality-value">{quality_label}</div>
            <div class="quality-slider-container">
                <input
                    type="range"
                    class="quality-slider"
                    min="1"
                    max="3"
                    value=move || slider_value().to_string()
                    on:input=move |ev| {
                        let value: i32 = event_target_value(&ev).parse().unwrap_or(2);
                        let level = quality_values.get((value - 1) as usize).unwrap_or(&"mid");
                        state.compression_level.set(level.to_string());
                    }
                />
            </div>
            <div class="quality-labels">
                <span>"Low"</span>
                <span>"Mid"</span>
                <span>"Max"</span>
            </div>
            <div class="quality-note">"Max = Smaller File • Low = Better Image Quality"</div>
        </div>
    }
}
