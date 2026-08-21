use chrono::Utc;
use uuid::Uuid;

pub struct File {
    pub id: Uuid,
    pub name: String,
    pub size_byte: usize,
    pub ext: String,
    pub added: chrono::DateTime<Utc>,
    pub created_by: String,
}
