use crate::server_functions::user::get_user_email;
use leptos::prelude::*;

#[component]
pub fn Profile() -> impl IntoView {
    let user = Resource::new(|| (), |_| get_user_email());

    view! {
        <h1>"Profile"</h1>
        <form method="post" action="/auth/logout">
            <Transition fallback=|| ()>
                {move || Suspend::new(async move {
                    match user.await {
                        Ok(email) => view! { <p>"Email: "{email}</p> }.into_any(),
                        Err(_) => view! { <p>"error fetching email"</p> }.into_any(),
                    }
                })}
            </Transition>
            <button type="submit" class="button-default">
                "logout"
            </button>
        </form>
    }
}
