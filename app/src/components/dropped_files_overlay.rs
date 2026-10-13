use crate::components::svg::{cross::CrossIcon, tick::TickIcon};
use domain::{files::UploadedFile, helpers::format_size};
use leptos::prelude::*;

#[component]
pub fn DroppedFilesOverlay(dropped_files: RwSignal<Vec<UploadedFile>>) -> impl IntoView {
    view! {
        <div
            class="drop-overlay-files"
            aria-hidden=move || { (dropped_files.get().is_empty()).then_some("true") }
            class:active=move || { !dropped_files.get().is_empty() }
        >
            <div class="drop-over-component">
                <For
                    each=move || {
                        let mut files = dropped_files.get();
                        files.sort_by_key(|f| !f.is_valid());
                        files
                    }
                    key=|state| state.id let(child)>
                    <div class="dropped-file">
                        <div class="dropped-file-name">
                            <p>
                                {
                                    if child.is_valid() {
                                        view! { <TickIcon class="file-valid" /> }.into_any()
                                    } else {
                                        view! { <CrossIcon class="file-invalid" /> }.into_any()
                                    }
                                }
                                {child.name()}
                            </p>
                            <button on:click=move |_| {
                                dropped_files
                                    .update(|n| {
                                        n.retain(|f| f.id != child.id);
                                    });
                            }>remove</button>
                        </div>
                        <small>{format_size(child.size())}</small>
                        {(!child.is_valid()).then(|| view! {
                        <div class="file-errors">
                            {child.errors().iter().map(|err| view! { <span>{err.to_string()}</span> }).collect_view()}
                        </div>
                         })}
                    </div>
                </For>
                <button
                    aria-hidden=move || { (dropped_files.get().is_empty()).then_some("true") }
                    on:click=move |_| { dropped_files.set(Vec::<UploadedFile>::new()) }
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}
