use leptos::prelude::*;

#[component]
pub fn SearchIcon(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" width="24" height="24">
            <g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="10" cy="10" r="6" />
                <path d="M15 15l6 6" />
                <path d="M10 7v6M7 10h6" stroke-width="1" />
            </g>
        </svg>
    }
}
