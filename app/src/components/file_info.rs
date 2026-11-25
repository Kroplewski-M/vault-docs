use domain::files::UserFile;
use leptos::prelude::*;

#[component]
pub fn FileInfo(file: UserFile) -> impl IntoView {
    view! { <p>{file.full_name}</p> }
}
