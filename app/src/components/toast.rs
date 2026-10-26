use std::time::Duration;

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

#[derive(Clone)]
pub struct Toast {
    id: u64,
    kind: ToastKind,
    message: String,
}

#[derive(Clone, Copy)]
pub struct Toasts {
    list: RwSignal<Vec<Toast>>,
    next_id: StoredValue<u64>,
}

impl Toasts {
    pub fn new() -> Self {
        Self {
            list: RwSignal::new(Vec::<Toast>::new()),
            next_id: StoredValue::new(0),
        }
    }
    pub fn push(&self, kind: ToastKind, message: impl Into<String>) {
        let id = self.next_id.get_value();
        self.next_id.set_value(id + 1);
        self.list.update(|l| {
            l.push(Toast {
                id,
                kind,
                message: message.into(),
            })
        });
        let this = *self;
        //only called from event handlers so this always runs client-side
        set_timeout(move || this.dismiss(id), Duration::from_secs(4));
    }
    pub fn success(&self, msg: impl Into<String>) {
        self.push(ToastKind::Success, msg.into());
    }
    pub fn error(&self, msg: impl Into<String>) {
        self.push(ToastKind::Error, msg.into());
    }
    pub fn info(&self, msg: impl Into<String>) {
        self.push(ToastKind::Info, msg.into());
    }
    pub fn dismiss(&self, id: u64) {
        self.list.update(|l| l.retain(|t| t.id != id));
    }
}

#[component]
pub fn Toaster() -> impl IntoView {
    let toasts = expect_context::<Toasts>();
    view! {
        <div class="toaster" role="status" aria-live="polite">
            <For each=move || toasts.list.get() key=|t| t.id let(t)>
                <div
                    class="toast"
                    class:toast-success=t.kind == ToastKind::Success
                    class:toast-error=t.kind == ToastKind::Error
                >
                    <p>{t.message}</p>
                    <button aria-label="Dismiss" on:click=move |_| toasts.dismiss(t.id)>
                        "×"
                    </button>
                </div>
            </For>
        </div>
    }
}
