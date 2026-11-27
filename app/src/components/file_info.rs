use std::path::Path;

use domain::files::UserFile;
use leptos::prelude::*;

use crate::components::svg::{md::MdIcon, pdf::PdfIcon, txt::TxtIcon};

#[component]
pub fn FileInfo(file: UserFile) -> impl IntoView {
    let file_ext = Path::new(&file.full_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    view! {
        <div class="file">
            {if file_ext == "pdf" {
                view! { <PdfIcon /> }.into_any()
            } else if file_ext == "txt" {
                view! { <TxtIcon /> }.into_any()
            } else if file_ext == "md" {
                view! { <MdIcon /> }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }} <p>{file.full_name}</p>
        </div>
    }
}
