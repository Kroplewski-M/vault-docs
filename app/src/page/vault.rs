use domain::files::UploadedFile;
use leptos::{ev, prelude::*};
use leptos_meta::Title;

use crate::components::dropped_files_overlay::DroppedFilesOverlay;

fn has_files(e: &ev::DragEvent) -> bool {
    let Some(items) = e.data_transfer().map(|dt| dt.items()) else {
        return false;
    };
    (0..items.length()).any(|i| items.get(i).is_some_and(|item| item.kind() == "file"))
}

#[component]
pub fn Vault() -> impl IntoView {
    // Counter, not a bool: dragenter/dragleave fire for every child element,
    // so a bool would flicker as the cursor moves over the page.
    let drag_depth = RwSignal::new(0i32);
    let dropped_files = RwSignal::new(Vec::<UploadedFile>::new());

    // Effect = client-only, so this never runs during SSR
    Effect::new(move |_| {
        let enter = window_event_listener(ev::dragenter, move |e| {
            if !has_files(&e) {
                return;
            }
            e.prevent_default();
            drag_depth.update(|d| *d += 1);
        });
        // Without preventDefault on dragover, the browser won't fire `drop`
        // and will open the file in the tab instead.
        let over = window_event_listener(ev::dragover, move |e| {
            e.prevent_default();
        });
        let leave = window_event_listener(ev::dragleave, move |e| {
            if !has_files(&e) {
                return;
            }
            drag_depth.update(|d| *d = (*d - 1).max(0));
        });
        let drop_h = window_event_listener(ev::drop, move |e| {
            e.prevent_default();
            drag_depth.set(0);
            let Some(list) = e.data_transfer().and_then(|dt| dt.files()) else {
                return;
            };
            let files: Vec<web_sys::File> =
                (0..list.length()).filter_map(|i| list.get(i)).collect();

            for file in files {
                leptos::task::spawn_local(async move {
                    let Ok(buf) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await
                    else {
                        return;
                    };
                    let bytes = js_sys::Uint8Array::new(&buf).to_vec();
                    dropped_files.update(|d| d.push(UploadedFile::new(file.name(), bytes)));
                });
            }
        });
        on_cleanup(move || {
            enter.remove();
            over.remove();
            leave.remove();
            drop_h.remove();
        });
    });
    view! {
        <Title formatter=|text| format!("{text} - My Vault") />
        <h1>"My Vault"</h1>
        <div
            class="drop-overlay"
            aria-hidden=move || { (drag_depth.get() == 0).then_some("true") }
            class:active=move || { drag_depth.get() > 0 }
        >
            <div class="drop-overlay_box">
                <p class="drop-overlay_title">"Drop files to upload"</p>
                <p class="drop-overlay_hint">"Release anywhere on the page"</p>
                <br />
            </div>
        </div>
        <DroppedFilesOverlay dropped_files=dropped_files />
    }
}
