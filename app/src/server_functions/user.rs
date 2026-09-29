use domain::models::CurrentUser;
use leptos::prelude::*;

pub type UserResource = Resource<Result<Option<CurrentUser>, ServerFnError>>;

#[server]
pub async fn get_user_email() -> Result<Option<String>, ServerFnError> {
    use services::users::UserService;

    let user = get_current_user().await;

    let user_service = expect_context::<UserService>();
    match user {
        Ok(Some(user)) => Ok(user_service.get_user_email(user.id).await?),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}
#[server]
pub async fn get_current_user() -> Result<Option<CurrentUser>, ServerFnError> {
    use axum::Extension;
    let user = leptos_axum::extract::<Option<Extension<CurrentUser>>>().await?;
    Ok(user.map(|Extension(u)| u))
}
