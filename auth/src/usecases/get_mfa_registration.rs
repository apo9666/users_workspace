use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use contracts::auth::{
    error::AuthError,
    mfa::{MfaRegistrationInput, MfaRegistrationOutput},
};

use crate::{
    entities::claims::Claims,
    ports::{for_auth_tokens::ForAuthTokens, user_repository::UserRepository},
};

pub struct GetMfaRegistrationUseCase {
    user_repository: Arc<dyn UserRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
}

impl GetMfaRegistrationUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
    ) -> Self {
        Self {
            user_repository,
            for_auth_tokens,
        }
    }

    pub async fn execute(
        &self,
        input: MfaRegistrationInput,
    ) -> Result<MfaRegistrationOutput, AuthError> {
        let claims = self
            .for_auth_tokens
            .validate_token(input.access_token, "access".to_string())
            .await
            .map_err(|_| AuthError::TokenValidationFailed)?;

        let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::UserNotFound)?;
        let user = self
            .user_repository
            .find_id(user_id)
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("time should go forward");
        let expires_in = 180;
        let exp = (since_the_epoch.as_secs() + expires_in) as usize;
        let mfa_registration = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "mfa_registration".to_string(),
                sub: user.id.to_string(),
                exp,
            })
            .await
            .map_err(|_| AuthError::MFATokenCreationFailed)?;

        Ok(MfaRegistrationOutput {
            mfa_registration,
            allowed_methods: vec!["totp".to_string(), "webauthn".to_string()],
            expires_in: expires_in as usize,
        })
    }
}
