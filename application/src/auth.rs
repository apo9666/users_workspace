use crate::port::credential_repository::{Credential, CredentialRepository};
use std::sync::Arc;

pub struct Auth {
    credential_repository: Arc<dyn CredentialRepository>,
}

impl Auth {
    pub fn new(credential_repository: Arc<dyn CredentialRepository>) -> Self {
        Self {
            credential_repository,
        }
    }

    pub async fn signup(&self, credential: &Credential) -> Result<(), Box<dyn std::error::Error>> {
        self.credential_repository.save(credential).await.unwrap();
        Ok(())
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(stored_password) = self
            .credential_repository
            .find_username(username.to_string())
            .await?
        {
            Ok(stored_password == password)
        } else {
            Ok(false)
        }
    }
}
