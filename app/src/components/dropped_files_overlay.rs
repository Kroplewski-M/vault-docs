use crate::components::svg::{cross::CrossIcon, tick::TickIcon};
use crate::page::vault::DroppedFile;
use crate::server_functions::files::upload_files;
use domain::helpers::format_size;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use web_sys::FormData;

#[component]
pub fn DroppedFilesOverlay(dropped_files: RwSignal<Vec<DroppedFile>>) -> impl IntoView {
    let uploading_files = RwSignal::new(false);
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
                            <Show when=move || !uploading_files.get()>
                                <button on:click=move |_| {
                                    dropped_files
                                        .update(|n| {
                                            n.retain(|f| f.meta.id != child.meta.id);
                                        });
                                }>remove</button>
                            </Show>

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
                <Show
                    when=move || !uploading_files.get()
                    fallback=move || {
                        let count = dropped_files.get().len();
                        view! {
                            <div class="upload-status" role="status" aria-live="polite">
                                <span class="upload-status-spinner" aria-hidden="true"></span>
                                <div>
                                    <p class="upload-status-title">
                                        {format!(
                                            "Uploading {count} file{}…",
                                            if count == 1 { "" } else { "s" },
                                        )}
                                    </p>
                                    <small>"Please keep this tab open until it finishes."</small>
                                </div>
                            </div>
                        }
                    }
                >
                    <div class="dropped-files_btn_container">
                        <button
                            aria-hidden=move || {
                                dropped_files.with(|f| f.is_empty()).then_some("true")
                            }
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
                            class:hidden=move || {
                                dropped_files.with(|f| f.iter().all(|x| !x.meta.is_valid()))
                            }
                            on:click=move |_| {
                                dropped_files.update(|f| f.retain(|file| file.meta.is_valid()));
                                let form = FormData::new().unwrap();
                                dropped_files
                                    .with(|files| {
                                        for f in files.iter().filter(|f| f.meta.is_valid()) {
                                            form.append_with_blob_and_filename(
                                                    "file",
                                                    &f.file,
                                                    &f.meta.name(),
                                                )
                                                .unwrap();
                                        }
                                    });
                                spawn_local(async move {
                                    uploading_files.set(true);
                                    if let Err(e) = upload_files(form.into()).await {
                                        leptos::logging::error!("upload failed: {e}");
                                    }
                                    uploading_files.set(false);
                                });
                            }
                        >
                            "Upload"
                        </button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
