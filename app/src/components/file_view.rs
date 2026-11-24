use leptos::prelude::*;

use crate::server_functions::files::get_file_metadata_for_user;

#[component]
pub fn FileView() -> impl IntoView {
    let files = Resource::new(|| (), |_| get_file_metadata_for_user());

    view! {
        <Suspense fallback=|| {
            view! { <p>"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                match files.await {
                    Ok(files) => {
                        view! {
                            <ul>
                                {files
                                    .into_iter()
                                    .map(|f| view! { <li>{f.full_name}</li> })
                                    .collect_view()}
                            </ul>
                        }
                            .into_any()
                    }
                    Err(e) => view! { <p>"Error: " {e.to_string()}</p> }.into_any(),
                }
            })}
        </Suspense>
    }
}
