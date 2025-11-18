use crate::{
    domain::{claims::Claims, credential::Credential},
    port::{
        credential_repository::CredentialRepository,
        for_auth_tokens::{self, ForAuthTokens},
    },
};
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Auth {
    credential_repository: Arc<dyn CredentialRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
}

impl Auth {
    pub fn new(
        credential_repository: Arc<dyn CredentialRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
    ) -> Self {
        Self {
            credential_repository,
            for_auth_tokens,
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
        _password: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(_stored_password) = self
            .credential_repository
            .find_username(username.to_string())
            .await?
        {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("time should go forward");
            let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
            let token = self
                .for_auth_tokens
                .create_token(Claims {
                    sub: username.to_string(),
                    exp,
                })
                .await?;
            self.for_auth_tokens.validate_token(&token).await?;
            return Ok(token);
        } else {
            Ok("".to_string())
        }
    }
}
