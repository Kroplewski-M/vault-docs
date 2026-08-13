use leptos::prelude::*;

#[component]
pub fn KeyIcon(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" width="24" height="24">
            <g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="8" cy="15" r="4" />
                <path d="M11 12l9-9" />
                <path d="M16 7l3 3" />
                <path d="M13 10l2.5 2.5" />
            </g>
        </svg>
    }
}
