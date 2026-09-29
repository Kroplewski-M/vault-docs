use leptos::prelude::*;

use crate::{
    components::{logo::Logo, svg::user::UserIcon},
    server_functions::user::UserResource,
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
                        Ok(Some(_u)) => {
                            view! {
                                <a href="/profile" class="profileIcon" aria-label="Profile">
                                    <UserIcon class="profileSvg" />
                                </a>
                            }
                                .into_any()
                        }
                        _ => view! { "" }.into_any(),
                    }
                })}
            </Transition>
        </nav>
    }
}
