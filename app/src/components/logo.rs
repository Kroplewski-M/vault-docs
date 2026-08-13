use leptos::prelude::*;

#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <a href="/" class="logo">
            <img src="/logo.svg" />
            <span>"Vault Docs"</span>
        </a>
    }
}
