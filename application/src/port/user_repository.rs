use crate::domain::user::User;
use async_trait::async_trait;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UserRepositoryError {
    ConnectionError(String),
}

impl fmt::Display for UserRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRepositoryError::ConnectionError(msg) => {
                write!(f, "Connection error: {}", msg)
            }
        }
    }
}

impl Error for UserRepositoryError {}

#[async_trait]
pub trait UserRepository {
    async fn save(&self, credential: User) -> Result<(), UserRepositoryError>;
    async fn find_username(&self, username: String) -> Result<Option<User>, UserRepositoryError>;
    async fn find_id(&self, id: uuid::Uuid) -> Result<Option<User>, UserRepositoryError>;
}
