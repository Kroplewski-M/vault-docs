use leptos::prelude::*;

#[component]
pub fn MdIcon(#[prop(optional, into)] class: String) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" width="24" height="24">
            <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
                <path
                    stroke-width="1.5"
                    d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8l-5-5z"
                />
                <path stroke-width="1.5" d="M14 3v5h5" />
                <g stroke-width="1.2">
                    <path d="M7.5 18v-5l2 3 2-3v5" />
                    <path d="M13.5 13v5h.75a2.5 2.5 0 0 0 0-5h-.75z" />
                </g>
            </g>
        </svg>
    }
}
