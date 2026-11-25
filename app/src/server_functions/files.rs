use domain::files::{FileUploadResult, UserFile};
#[cfg(feature = "ssr")]
use domain::models::CurrentUser;
use leptos::prelude::*;
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

#[cfg(feature = "ssr")]
pub async fn require_user() -> Result<CurrentUser, ServerFnError> {
    use crate::server_functions::user::get_current_user;

    get_current_user()
        .await?
        .ok_or_else(|| ServerFnError::new("unauthorized"))
}

#[server(input = MultipartFormData)]
pub async fn upload_files(data: MultipartData) -> Result<Vec<FileUploadResult>, ServerFnError> {
    use services::files::FileService;

    let user = require_user().await?;

    let mut results = Vec::<FileUploadResult>::new();

    let file_service = expect_context::<FileService>();

    let mut data = data.into_inner().unwrap();

    loop {
        let field = match data.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return Err(ServerFnError::new(format!("multipart error: {e}"))),
        };
        let id = field.name().unwrap_or_default().to_string();
        let name = field.file_name().unwrap_or_default().to_string();

        let (error, file) = match field.bytes().await {
            Err(e) => (Some(format!("failed to read file: {e}")), None),
            Ok(bytes) => match file_service
                .upload_file(user.id, name.clone(), &bytes)
                .await
            {
                Ok(meta) => (
                    None,
                    Some(UserFile {
                        id: meta.id,
                        full_name: format!("{}.{}", meta.original_name, meta.ext),
                        size: meta.size,
                    }),
                ),
                Err(e) => (Some(e.to_string()), None),
            },
        };

        results.push(FileUploadResult {
            id,
            name,
            error,
            file,
        });
    }
    Ok(results)
}

#[server]
pub async fn get_file_metadata_for_user() -> Result<Vec<UserFile>, ServerFnError> {
    use services::files::FileService;

    let user = require_user().await?;

    let file_service = expect_context::<FileService>();

    file_service
        .fetch_user_files(&user.id)
        .await
        .map_err(ServerFnError::new)
}
