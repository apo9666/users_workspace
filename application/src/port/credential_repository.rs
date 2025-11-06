use super::repository_error::RepositoryError;
use async_trait::async_trait;

pub struct Credential {
    pub username: String,
    pub password: String,
}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn save(&self, credential: &Credential) -> Result<(), RepositoryError>;
    async fn find_username(&self, username: String) -> Result<Option<String>, RepositoryError>;
}
