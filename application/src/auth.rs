use crate::{
    domain::{claims::Claims, credential::Credential},
    port::{credential_repository::CredentialRepository, for_auth_tokens::ForAuthTokens},
};
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
}

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
        password: &str,
    ) -> Result<LoginResult, Box<dyn std::error::Error>> {
        let Some(hashed_password) = self
            .credential_repository
            .find_username(username.to_string())
            .await?
        else {
            return Err("Invalid username or password".to_string().into());
        };

        if !verify(password, &hashed_password)? {
            return Err("Invalid username or password".into());
        }

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        let exp = (since_the_epoch.as_secs() + 604800) as usize; // 7 days from now
        let refresh_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "refresh".to_string(),
                sub: username.to_string(),
                exp,
            })
            .await?;

        let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
        let access_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "access".to_string(),
                sub: username.to_string(),
                exp,
            })
            .await?;
        self.for_auth_tokens.validate_token(&refresh_token).await?;

        return Ok(LoginResult {
            access_token,
            refresh_token,
        });
    }
}
