use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct File {
    pub id: Uuid,
    pub name: String,
    pub size_byte: usize,
    pub ext: String,
    pub added: chrono::DateTime<Utc>,
    pub created_by: String,
}
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}
