use leptos::prelude::*;

#[component]
pub fn Logo() -> impl IntoView {
    view! {
        <a href="/" class="logo">
            <img src="/logo.svg" alt="" />
            <span>"Vault Docs"</span>
        </a>
    }
}
