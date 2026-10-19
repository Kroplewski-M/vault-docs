use crate::components::svg::{cross::CrossIcon, tick::TickIcon};
use crate::page::vault::DroppedFile;
use domain::{files::UploadedFile, helpers::format_size};
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos::server_fn::codec::{MultipartData, MultipartFormData};
use web_sys::FormData;

#[server(input = MultipartFormData)]
pub async fn upload_files(data: MultipartData) -> Result<(), ServerFnError> {
    let mut data = data.into_inner().unwrap();
    while let Ok(Some(field)) = data.next_field().await {
        let name = field.file_name().unwrap_or_default().to_string();
        let bytes = field.bytes().await.map_err(ServerFnError::new)?;
        //Revalidate the files on the server
        let file = UploadedFile::new(name, &bytes);
        if file.is_valid() {
            leptos::logging::log!("received {} ({} bytes)", file.name(), file.size());
        }
    }
    Ok(())
}

#[component]
pub fn DroppedFilesOverlay(dropped_files: RwSignal<Vec<DroppedFile>>) -> impl IntoView {
    view! {
        <div
            class="drop-overlay-files"
            aria-hidden=move || dropped_files.with(|f| f.is_empty()).then_some("true")
            class:active=move || !dropped_files.with(|f| f.is_empty())
        >
            <div class="drop-over-component">
                <For
                    each=move || {
                        let mut files = dropped_files.get();
                        files.sort_by_key(|f| !f.meta.is_valid());
                        files
                    }
                    key=|state| state.meta.id
                    let(child)
                >
                    <div class="dropped-file">
                        <div class="dropped-file-name">
                            <p>
                                {if child.meta.is_valid() {
                                    view! { <TickIcon class="file-valid" /> }.into_any()
                                } else {
                                    view! { <CrossIcon class="file-invalid" /> }.into_any()
                                }} {child.meta.name()}
                            </p>
                            <button on:click=move |_| {
                                dropped_files
                                    .update(|n| {
                                        n.retain(|f| f.meta.id != child.meta.id);
                                    });
                            }>remove</button>
                        </div>
                        <small>{format_size(child.meta.size())}</small>
                        {(!child.meta.is_valid())
                            .then(|| {
                                view! {
                                    <div class="file-errors">
                                        {child
                                            .meta
                                            .errors()
                                            .iter()
                                            .map(|err| view! { <span>{err.to_string()}</span> })
                                            .collect_view()}
                                    </div>
                                }
                            })}
                    </div>
                </For>
                <div class="dropped-files_btn_container">
                    <button
                        aria-hidden=move || dropped_files.with(|f| f.is_empty()).then_some("true")
                        on:click=move |_| { dropped_files.set(Vec::<DroppedFile>::new()) }
                        class="button-default"
                    >
                        "Cancel"
                    </button>
                    <button
                        aria-hidden=move || {
                            dropped_files
                                .with(|f| f.iter().all(|x| !x.meta.is_valid()))
                                .then_some("true")
                        }
                        class="button-success"
                        class:hidden=move || dropped_files.with(|f| f.iter().all(|x| !x.meta.is_valid()))
                        on:click=move |_| {
                            let form = FormData::new().unwrap();
                            dropped_files
                                .with(|files| {
                                    for f in files.iter().filter(|f| f.meta.is_valid()) {
                                        form.append_with_blob_and_filename("file", &f.file, &f.meta.name())
                                            .unwrap();
                                    }
                                });
                            spawn_local(async move {
                                if let Err(e) = upload_files(form.into()).await {
                                    leptos::logging::error!("upload failed: {e}");
                                }
                            });
                        }
                    >
                        "Upload"
                    </button>
                </div>
            </div>
        </div>
    }
}
