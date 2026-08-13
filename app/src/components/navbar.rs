use leptos::prelude::*;

use crate::components::logo::Logo;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <nav class="nav">
            <Logo />
        </nav>
    }
}
