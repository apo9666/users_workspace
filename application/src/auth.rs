use crate::port::credential_repository::{Credential, CredentialRepository};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Auth {
    credential_repository: Arc<dyn CredentialRepository>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    // aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    // iat: usize, // Optional. Issued at (as UTC timestamp)
    // iss: String, // Optional. Issuer
    // nbf: usize,  // Optional. Not Before (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
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
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(stored_password) = self
            .credential_repository
            .find_username(username.to_string())
            .await?
        {
            let valid = verify(password, &stored_password)?;
            if valid {
                let my_claims = Claims {
                    sub: "b@b.com".to_owned(),
                    exp: 10000000000,
                };
                let token = encode(
                    &Header::default(),
                    &my_claims,
                    &EncodingKey::from_secret("secret".as_ref()),
                )?;
                return Ok(token);
            }
            Ok("".to_string())
        } else {
            Ok("".to_string())
        }
    }
}
