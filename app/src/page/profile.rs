use leptos::prelude::*;

#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <h1>"Profile"</h1>
        <form method="post" action="/auth/logout">
            <button type="submit" class="button-default">"logout"</button>
        </form>
    }
}
