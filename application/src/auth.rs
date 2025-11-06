use std::sync::Arc;
use bcrypt::{DEFAULT_COST, hash, verify};
use crate::port::credential_repository::{Credential, CredentialRepository};

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
        let cred = Credential {
            username: credential.username.clone(),
            password: hash(&credential.password, DEFAULT_COST)?,
        };

        self.credential_repository.save(&cred).await.unwrap();
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
            let valid = verify(password, &stored_password)?;
            Ok(valid)
        } else {
            Ok(false)
        }
    }
}
