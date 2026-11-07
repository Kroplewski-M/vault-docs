use domain::files::FileUploadResult;
use leptos::prelude::*;
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

#[server(input = MultipartFormData)]
pub async fn upload_files(data: MultipartData) -> Result<Vec<FileUploadResult>, ServerFnError> {
    use crate::server_functions::user::get_current_user;
    use services::files::FileService;

    let Some(user) = get_current_user().await? else {
        return Err(ServerFnError::new("unauthorized"));
    };

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

        let error = match field.bytes().await {
            Err(e) => Some(format!("failed to read file: {e}")),
            Ok(bytes) => file_service
                .upload_file(user.id, name.clone(), &bytes)
                .await
                .err()
                .map(|e| e.to_string()),
        };

        results.push(FileUploadResult { id, name, error });
    }
    Ok(results)
}
