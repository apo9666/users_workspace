use application::port::credential_repository::{Credential, CredentialRepository};
use application::port::repository_error::RepositoryError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type SharedCredentials = Arc<Mutex<HashMap<String, String>>>;
pub struct MemoryCredentialRepository {
    credentials: SharedCredentials,
}

impl MemoryCredentialRepository {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CredentialRepository for MemoryCredentialRepository {
    async fn save(&self, credential: &Credential) -> Result<(), RepositoryError> {
        let mut repositories = self
            .credentials
            .lock()
            .map_err(|e| RepositoryError::ConnectionError(format!("Mutex poisoned: {}", e)))?;
        repositories.insert(credential.username.clone(), credential.password.clone());
        Ok(())
    }

    async fn find_username(&self, username: String) -> Result<Option<String>, RepositoryError> {
        let repositories = self
            .credentials
            .lock()
            .map_err(|e| RepositoryError::ConnectionError(format!("Mutex poisoned: {}", e)))?;
        Ok(repositories.get(&username).cloned())
    }
}
