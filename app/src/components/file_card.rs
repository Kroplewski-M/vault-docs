use domain::models::File;
use leptos::prelude::*;

#[component]
pub fn FileCard(file: File) -> impl IntoView {
    view! {
        <div class="file">
            <h3>{file.name}</h3>
        </div>
    }
}
