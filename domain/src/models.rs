use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[repr(i16)] // SMALLINT
pub enum EncVersion {
    /// AES-256-GCM
    V1 = 1,
}
#[derive(Clone, Debug)]
pub struct DataKey {
    pub plain_text: Vec<u8>,
    pub encrypted: Vec<u8>,
}
