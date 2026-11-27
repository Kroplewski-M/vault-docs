use domain::files::UserFile;
use leptos::prelude::*;

use crate::components::{file_info::FileInfo, svg::empty_files::EmptyFilesIcon};

#[component]
pub fn FileView(files: Resource<Result<Vec<UserFile>, ServerFnError>>) -> impl IntoView {
    view! {
        <Suspense fallback=|| {
            view! { <p>"Loading..."</p> }
        }>
            {move || Suspend::new(async move {
                match files.await {
                    Ok(files) => {
                        view! {
                            {if !files.is_empty() {
                                view! {
                                    <div class="files">
                                        {files
                                            .into_iter()
                                            .map(|f| view! { <FileInfo file=f /> })
                                            .collect_view()
                                            .into_any()}
                                    </div>
                                }
                                    .into_any()
                            } else {

                                view! {
                                    <div class="empty_files">
                                        <EmptyFilesIcon />
                                        <p>"No Files Added"</p>
                                    </div>
                                }
                                    .into_any()
                            }}
                        }
                            .into_any()
                    }
                    Err(e) => view! { <p>"Error: " {e.to_string()}</p> }.into_any(),
                }
            })}
        </Suspense>
    }
}
