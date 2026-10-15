use domain::files::FileFormat;
use leptos::prelude::*;

#[component]
pub fn DroppingFilesOverlay() -> impl IntoView {
    view! {
        <div class="drop-overlay_box">
            <p class="drop-overlay_title">"Drop files to upload"</p>
            <p class="drop-overlay_hint">"Release anywhere on the page"</p>
            <br />
            <p class="drop-overlay_hint">"Supported files: "</p>
            <div class="drop-ovlay-files">
                {FileFormat::ALL
                    .iter()
                    .map(|ex| view! { <span>{ex.extension()}</span> })
                    .collect_view()}
            </div>
        </div>
    }
}
