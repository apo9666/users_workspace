use crate::entities::user::User;
use crate::ports::user_repository::UserRepository;
use async_trait::async_trait;
use contracts::auth::error::UserRepositoryError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type SharedUsers = Arc<Mutex<HashMap<String, User>>>;
pub struct MemoryUserRepository {
    users: SharedUsers,
}

impl MemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserRepository for MemoryUserRepository {
    async fn save(&self, user: User) -> Result<(), UserRepositoryError> {
        let mut repositories = self
            .users
            .lock()
            .map_err(|e| UserRepositoryError::ConnectionError(format!("Mutex poisoned: {}", e)))?;
        repositories.insert(user.username.clone(), user.clone());
        Ok(())
    }

    async fn find_username(&self, username: String) -> Result<Option<User>, UserRepositoryError> {
        let repositories = self
            .users
            .lock()
            .map_err(|e| UserRepositoryError::ConnectionError(format!("Mutex poisoned: {}", e)))?;
        Ok(repositories.get(&username).cloned())
    }

    async fn find_id(&self, id: uuid::Uuid) -> Result<Option<User>, UserRepositoryError> {
        let repositories = self
            .users
            .lock()
            .map_err(|e| UserRepositoryError::ConnectionError(format!("Mutex poisoned: {}", e)))?;
        for user in repositories.values() {
            if user.id == id {
                return Ok(Some(user.clone()));
            }
        }
        Ok(None)
    }
}
