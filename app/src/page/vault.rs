use leptos::{ev, prelude::*};
use leptos_meta::Title;

#[component]
pub fn Vault() -> impl IntoView {
    // Counter, not a bool: dragenter/dragleave fire for every child element,
    // so a bool would flicker as the cursor moves over the page.
    let drag_depth = RwSignal::new(0i32);
    let dropped_files = RwSignal::new(Vec::<(u64, String)>::new());
    let next_id = StoredValue::new(0u64);

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
        let over = window_event_listener(ev::dragover, |e| e.prevent_default());
        let leave = window_event_listener(ev::dragleave, move |_| {
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

            dropped_files.update(|d| {
                d.extend(files.iter().map(|f| {
                    let id = next_id.get_value();
                    next_id.set_value(id + 1);
                    (id, f.name())
                }))
            });
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
            </div>
        </div>

        <ul class="upload-list">
            <For each=move || dropped_files.get() key=|(id, _)| *id let:entry>
                <li>{entry.1}</li>
            </For>
        </ul>
    }
}
fn has_files(e: &ev::DragEvent) -> bool {
    let Some(items) = e.data_transfer().map(|dt| dt.items()) else {
        return false;
    };
    (0..items.length()).any(|i| items.get(i).is_some_and(|item| item.kind() == "file"))
}
