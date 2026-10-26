use std::thread::sleep;
use std::time::Duration;

use domain::files::UploadedFile;
use leptos::prelude::*;
use leptos::server_fn::codec::{MultipartData, MultipartFormData};

use crate::server_functions::user::get_current_user;

#[server(input = MultipartFormData)]
pub async fn upload_files(data: MultipartData) -> Result<(), ServerFnError> {
    let Some(user) = get_current_user().await? else {
        return Err(ServerFnError::new("unauthorized"));
    };

    let mut data = data.into_inner().unwrap();
    while let Ok(Some(field)) = data.next_field().await {
        let name = field.file_name().unwrap_or_default().to_string();
        let bytes = field.bytes().await.map_err(ServerFnError::new)?;
        //Revalidate the files on the server
        let file = UploadedFile::new(name, &bytes);
        if file.is_valid() {
            leptos::logging::log!("received {} ({} bytes)", file.name(), file.size());
        }
    }
    sleep(Duration::from_secs(10));
    Ok(())
}
