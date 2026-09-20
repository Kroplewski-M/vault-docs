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
    pub async fn create_user_if_missing(&self, id: Uuid, email: &str) -> Result<()> {
        self.repo
            .create_user_if_missing(id, email)
            .await
            .map_err(internal)
    }
}
