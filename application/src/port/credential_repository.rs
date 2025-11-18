use crate::domain::credential::Credential;
use async_trait::async_trait;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CredentialRepositoryError {
    ConnectionError(String),
}

impl fmt::Display for CredentialRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialRepositoryError::ConnectionError(msg) => {
                write!(f, "Connection error: {}", msg)
            }
        }
    }
}

impl Error for CredentialRepositoryError {}

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn save(&self, credential: &Credential) -> Result<(), CredentialRepositoryError>;
    async fn find_username(
        &self,
        username: String,
    ) -> Result<Option<String>, CredentialRepositoryError>;
}
