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
