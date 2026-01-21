use crate::entities::user::User;
use async_trait::async_trait;
use contracts::auth::error::UserRepositoryError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, credential: User) -> Result<(), UserRepositoryError>;
    async fn find_username(&self, username: String) -> Result<Option<User>, UserRepositoryError>;
    async fn find_id(&self, id: uuid::Uuid) -> Result<Option<User>, UserRepositoryError>;
}
