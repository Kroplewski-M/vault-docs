use domain::{helpers::format_size, models::PendingFileUpload};
use leptos::prelude::*;

#[component]
pub fn DroppedFilesOverlay(dropped_files: RwSignal<Vec<PendingFileUpload>>) -> impl IntoView {
    view! {
        <div
            class="drop-overlay-files"
            aria-hidden=move || { (dropped_files.get().is_empty()).then_some("true") }
            class:active=move || { !dropped_files.get().is_empty() }
        >
            <div class="drop-over-component">
                <For each=move || dropped_files.get() key=|state| state.id let(child)>
                    <div class="dropped-file">
                        <div class="dropped-file-name">
                            <p>{child.file.name()}</p>
                            <button on:click=move |_| {
                                dropped_files
                                    .update(|n| {
                                        n.retain(|f| f.id != child.id);
                                    });
                            }>remove</button>
                        </div>
                        <small>{format_size(child.file.size())}</small>
                    </div>
                </For>
                <button
                    aria-hidden=move || { (dropped_files.get().is_empty()).then_some("true") }
                    on:click=move |_| { dropped_files.set(Vec::<PendingFileUpload>::new()) }
                >
                    "Cancel"
                </button>
            </div>
        </div>
    }
}
