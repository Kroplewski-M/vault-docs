use leptos::prelude::*;

use crate::{
    UserResource,
    components::{logo::Logo, svg::user::UserIcon},
};

#[component]
pub fn Navbar() -> impl IntoView {
    let user = expect_context::<UserResource>();

    view! {
        <nav class="nav">
            <Logo />
        <Transition fallback=|| ()>
                {move || Suspend::new(async move {
                    match user.await {
                        Ok(Some(_u)) => view! { <a href="/profile" class="profileIcon"><UserIcon class="profileSvg"/></a> }.into_any(),
                        _ => view!{""}.into_any(),
                    }
                })}
            </Transition>
        </nav>
    }
}
