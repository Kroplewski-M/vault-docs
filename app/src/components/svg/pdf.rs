use leptos::prelude::*;

#[component]
pub fn PdfIcon(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" width="24" height="24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
                <path
                    stroke-width="1.5"
                    d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8l-5-5z"
                />
                <path stroke-width="1.5" d="M14 3v5h5" />
                <g stroke-width="1.2">
                    <path d="M7.5 18v-5h1.25a1.5 1.5 0 0 1 0 3H7.5" />
                    <path d="M11 13v5h.75a2.5 2.5 0 0 0 0-5H11z" />
                    <path d="M17 13h-2v5" />
                    <path d="M15 15.5h1.5" />
                </g>
            </g>
        </svg>
    }
}
