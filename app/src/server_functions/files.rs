use leptos::prelude::*;
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

#[server(input = MultipartFormData)]
pub async fn upload_files(data: MultipartData) -> Result<(), ServerFnError> {
    use crate::server_functions::user::get_current_user;
    use domain::files::UploadedFile;
    use services::files::FileService;

    let Some(user) = get_current_user().await? else {
        return Err(ServerFnError::new("unauthorized"));
    };

    let file_service = expect_context::<FileService>();
    let mut data = data.into_inner().unwrap();
    while let Ok(Some(field)) = data.next_field().await {
        let name = field.file_name().unwrap_or_default().to_string();
        let bytes = field.bytes().await.map_err(ServerFnError::new)?;
        //Revalidate the files on the server
        let file = UploadedFile::new(name, &bytes);
        if file.is_valid() {
            leptos::logging::log!("received {} ({} bytes)", file.name(), file.size());
            file_service
                .upload_file(user.id, file.name(), &bytes)
                .await?;
        }
    }
    Ok(())
}
