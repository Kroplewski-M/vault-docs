use chrono::{Duration, Utc};
use db::sessions::SessionRepo;
use uuid::Uuid;

use crate::{Result, error::internal};

pub const SESSION_TTL_DAYS: i64 = 7;

#[derive(Clone)]
pub struct SessionService {
    repo: SessionRepo,
}

impl SessionService {
    pub fn new(repo: SessionRepo) -> Self {
        Self { repo }
    }
    pub async fn create(&self, user_id: Uuid) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::days(SESSION_TTL_DAYS);
        self.repo
            .create(id, user_id, expires_at)
            .await
            .map_err(internal)?;
        Ok(id)
    }
    pub async fn user_id_for_session(&self, session_id: Uuid) -> Result<Option<Uuid>> {
        self.repo
            .user_id_for_session(session_id)
            .await
            .map_err(internal)
    }
    pub async fn delete(&self, session_id: Uuid) -> Result<()> {
        self.repo.delete(session_id).await.map_err(internal)
    }
}
