use application::domain::credential::Credential;
use application::port::credential_repository::{CredentialRepository, CredentialRepositoryError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type SharedCredentials = Arc<Mutex<HashMap<String, Credential>>>;
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
    async fn save(&self, credential: Credential) -> Result<(), CredentialRepositoryError> {
        let mut repositories = self.credentials.lock().map_err(|e| {
            CredentialRepositoryError::ConnectionError(format!("Mutex poisoned: {}", e))
        })?;
        repositories.insert(credential.username.clone(), credential.clone());
        Ok(())
    }

    async fn find_username(
        &self,
        username: String,
    ) -> Result<Option<Credential>, CredentialRepositoryError> {
        let repositories = self.credentials.lock().map_err(|e| {
            CredentialRepositoryError::ConnectionError(format!("Mutex poisoned: {}", e))
        })?;
        Ok(repositories.get(&username).cloned())
    }
}
