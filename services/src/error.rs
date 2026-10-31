use core::fmt;

use domain::files::UploadError;

#[derive(Debug)]
pub enum Error {
    Internal(String),
    InvalidFile(Vec<UploadError>),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Internal(msg) => write!(f, "Internal error: {msg}"),
            Error::InvalidFile(errors) => write!(
                f,
                "Invalid file: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn internal(e: impl fmt::Display) -> Error {
    Error::Internal(e.to_string())
}
