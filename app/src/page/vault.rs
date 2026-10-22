use domain::files::{FileFormat, UploadedFile};
use leptos::{ev, prelude::*};
use leptos_meta::Title;

use crate::components::{
    dropped_files_overlay::DroppedFilesOverlay, dropping_files_overlay::DroppingFilesOverlay,
};

#[derive(Clone)]
pub struct DroppedFile {
    pub meta: UploadedFile,
    pub file: web_sys::File,
}

fn has_files(e: &ev::DragEvent) -> bool {
    let Some(items) = e.data_transfer().map(|dt| dt.items()) else {
        return false;
    };
    (0..items.length()).any(|i| items.get(i).is_some_and(|item| item.kind() == "file"))
}

fn add_files(list: web_sys::FileList, dropped_files: RwSignal<Vec<DroppedFile>>) {
    let files: Vec<web_sys::File> = (0..list.length()).filter_map(|i| list.get(i)).collect();

    for file in files {
        // Check the size before reading, so huge files never get loaded into memory
        let size = file.size() as usize;
        if UploadedFile::exceeds_max_size(size) {
            dropped_files.update(|d| {
                d.push(DroppedFile {
                    meta: UploadedFile::oversized(file.name(), size),
                    file,
                })
            });
            continue;
        }
        leptos::task::spawn_local(async move {
            let Ok(buf) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await else {
                return;
            };
            let bytes = js_sys::Uint8Array::new(&buf).to_vec();
            dropped_files.update(|d| {
                d.push(DroppedFile {
                    meta: UploadedFile::new(file.name(), &bytes),
                    file,
                })
            });
        });
    }
}
#[component]
pub fn Vault() -> impl IntoView {
    // Counter, not a bool: dragenter/dragleave fire for every child element,
    // so a bool would flicker as the cursor moves over the page.
    let drag_depth = RwSignal::new(0i32);
    let dropped_files = RwSignal::new(Vec::<DroppedFile>::new());

    let file_input = NodeRef::<leptos::html::Input>::new();
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
            add_files(list, dropped_files);
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
        <button
            class="button-default"
            on:click=move |_| {
                if let Some(input) = file_input.get() {
                    input.click();
                }
            }
        >
            "Uplaod Files"
        </button>
        <input
            type="file"
            multiple
            hidden
            aria-hidden
            node_ref=file_input
            accept=move || {
                FileFormat::ALL.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
            }
            on:change=move |ev| {
                let input = event_target::<web_sys::HtmlInputElement>(&ev);
                if let Some(list) = input.files() {
                    add_files(list, dropped_files);
                }
                input.set_value("");
            }
        />
        <div
            class="drop-overlay"
            aria-hidden=move || { (drag_depth.get() == 0).then_some("true") }
            class:active=move || { drag_depth.get() > 0 }
        >
            <DroppingFilesOverlay />
        </div>
        <DroppedFilesOverlay dropped_files=dropped_files />
    }
}
