use db::users::UserRepo;
use uuid::Uuid;

use crate::error::{Result, internal};

#[derive(Clone)]
pub struct UserService {
    repo: UserRepo,
}

impl UserService {
    pub fn new(repo: UserRepo) -> Self {
        Self { repo }
    }
    pub async fn upsert_from_sign_in(&self, id: Uuid, email: &str) -> Result<()> {
        self.repo.upsert(id, email).await.map_err(internal)
    }
}
