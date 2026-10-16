use crate::components::svg::{cross::CrossIcon, tick::TickIcon};
use domain::{files::UploadedFile, helpers::format_size};
use leptos::logging::log;
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
        let file = UploadedFile::new(name, bytes.to_vec());
        if file.is_valid() {
            log!("received {} ({} bytes)", file.name(), file.size());
        }
    }
    Ok(())
}

#[component]
pub fn DroppedFilesOverlay(dropped_files: RwSignal<Vec<UploadedFile>>) -> impl IntoView {
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
                        files.sort_by_key(|f| !f.is_valid());
                        files
                    }
                    key=|state| state.id
                    let(child)
                >
                    <div class="dropped-file">
                        <div class="dropped-file-name">
                            <p>
                                {if child.is_valid() {
                                    view! { <TickIcon class="file-valid" /> }.into_any()
                                } else {
                                    view! { <CrossIcon class="file-invalid" /> }.into_any()
                                }} {child.name()}
                            </p>
                            <button on:click=move |_| {
                                dropped_files
                                    .update(|n| {
                                        n.retain(|f| f.id != child.id);
                                    });
                            }>remove</button>
                        </div>
                        <small>{format_size(child.size())}</small>
                        {(!child.is_valid())
                            .then(|| {
                                view! {
                                    <div class="file-errors">
                                        {child
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
                        on:click=move |_| { dropped_files.set(Vec::<UploadedFile>::new()) }
                        class="button-default"
                    >
                        "Cancel"
                    </button>
                    <button
                        aria-hidden=move || {
                            dropped_files
                                .with(|f| f.iter().all(|x| !x.is_valid()))
                                .then_some("true")
                        }
                        class="button-success"
                        class:hidden=move || dropped_files.with(|f| f.iter().all(|x| !x.is_valid()))
                        on:click=move |_| {
                            let form = FormData::new().unwrap();
                            dropped_files
                                .with(|files| {
                                    for f in files.iter().filter(|f| f.is_valid()) {
                                        let arr = js_sys::Array::of1(
                                            &js_sys::Uint8Array::from(f.bytes()),
                                        );
                                        let blob = web_sys::Blob::new_with_u8_array_sequence(&arr)
                                            .unwrap();
                                        form.append_with_blob_and_filename("file", &blob, &f.name())
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
