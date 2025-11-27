use crate::{
    domain::{claims::Claims, credential::Credential},
    port::{
        credential_repository::CredentialRepository, for_auth_tokens::ForAuthTokens,
        for_totp::ForTotp,
    },
};
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LoginResult {
    pub mfa_token: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

pub struct Auth {
    credential_repository: Arc<dyn CredentialRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
    for_totp: Arc<dyn ForTotp>,
}

impl Auth {
    pub fn new(
        credential_repository: Arc<dyn CredentialRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
        for_totp: Arc<dyn ForTotp>,
    ) -> Self {
        Self {
            credential_repository,
            for_auth_tokens,
            for_totp,
        }
    }

    pub async fn signup(&self, credential: &Credential) -> Result<(), Box<dyn std::error::Error>> {
        let cred = Credential {
            username: credential.username.clone(),
            password: hash(&credential.password, DEFAULT_COST)?,
            otp_secret: None,
        };

        self.credential_repository.save(cred).await.unwrap();
        Ok(())
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<LoginResult, Box<dyn std::error::Error>> {
        let Some(credential) = self
            .credential_repository
            .find_username(username.clone())
            .await?
        else {
            return Err("Invalid username or password".to_string().into());
        };

        if !verify(password, &credential.password)? {
            return Err("Invalid username or password".into());
        }

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");

        if credential.otp_secret.is_some() {
            let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
            let mfa_token = self
                .for_auth_tokens
                .create_token(Claims {
                    token_type: "mfa".to_string(),
                    sub: username.clone(),
                    exp,
                })
                .await?;

            return Ok(LoginResult {
                mfa_token: Some(mfa_token),
                access_token: None,
                refresh_token: None,
            });
        }

        let exp = (since_the_epoch.as_secs() + 604800) as usize; // 7 days from now
        let refresh_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "refresh".to_string(),
                sub: username.clone(),
                exp,
            })
            .await?;

        let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
        let access_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "access".to_string(),
                sub: username.clone(),
                exp,
            })
            .await?;

        return Ok(LoginResult {
            mfa_token: None,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
        });
    }

    pub async fn mfa_totp_setup(
        &self,
        mfa_token: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let claims = self
            .for_auth_tokens
            .validate_token(mfa_token, "access".to_string())
            .await?;

        let Some(mut credential) = self.credential_repository.find_username(claims.sub).await?
        else {
            return Err("Invalid username or password".to_string().into());
        };

        let (secret, auth_url) = self
            .for_totp
            .auth_url(credential.username.clone(), "TODO_ISSUER".to_string())
            .await?;

        credential.otp_secret = Some(secret);
        self.credential_repository.save(credential).await?;

        Ok(auth_url)
    }
}
