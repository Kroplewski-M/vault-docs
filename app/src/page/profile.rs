use leptos::prelude::*;

#[server]
async fn get_user_email() -> Result<Option<String>, ServerFnError> {
    use crate::get_current_user;
    use services::users::UserService;

    let user = get_current_user().await;

    let user_service = expect_context::<UserService>();
    match user {
        Ok(Some(user)) => Ok(user_service.get_user_email(user.id).await?),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

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
