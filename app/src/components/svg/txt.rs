use leptos::prelude::*;

#[component]
pub fn TxtIcon(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" width="24" height="24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
                <path
                    stroke-width="1.5"
                    d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8l-5-5z"
                />
                <path stroke-width="1.5" d="M14 3v5h5" />
                <g stroke-width="1.2">
                    <path d="M6.5 13h3M8 13v5" />
                    <path d="M10.5 13l3 5M13.5 13l-3 5" />
                    <path d="M14.5 13h3M16 13v5" />
                </g>
            </g>
        </svg>
    }
}
