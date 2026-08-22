use leptos::prelude::*;

#[component]
pub fn EmptyFilesIcon(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" width="24" height="24">
            <g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                <path d="M3 7.5c0-.55.45-1 1-1h4.5l1.5 2h9c.55 0 1 .45 1 1v9.5c0 .55-.45 1-1 1H4c-.55 0-1-.45-1-1V7.5Z" />
            </g>
        </svg>
    }
}
