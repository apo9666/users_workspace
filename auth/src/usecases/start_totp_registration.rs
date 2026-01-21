use crate::entities::totp::TOTP_REG_STATE;
use crate::ports::for_auth_tokens::ForAuthTokens;
use crate::ports::for_totp::ForTotp;
use crate::ports::hsm_store::HSMStore;
use crate::ports::user_repository::UserRepository;
use contracts::auth::error::AuthError;
use contracts::auth::totp::{TOTPStartRegistrationInput, TOTPStartRegistrationOutput};
use std::sync::Arc;

pub struct StartTOTPRegistrationUseCase {
    user_repository: Arc<dyn UserRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
    for_totp: Arc<dyn ForTotp>,
    hsm_store: Arc<dyn HSMStore>,
}

impl StartTOTPRegistrationUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
        for_totp: Arc<dyn ForTotp>,
        hsm_store: Arc<dyn HSMStore>,
    ) -> Self {
        Self {
            user_repository,
            for_auth_tokens,
            for_totp,
            hsm_store,
        }
    }

    pub async fn execute(
        &self,
        input: TOTPStartRegistrationInput,
    ) -> Result<TOTPStartRegistrationOutput, AuthError> {
        let claims = self
            .for_auth_tokens
            .validate_token(input.mfa_token, "mfa_registration".to_string())
            .await
            .map_err(|_| AuthError::TokenValidationFailed)?;

        let Some(user) = self
            .user_repository
            .find_username(claims.sub)
            .await
            .map_err(AuthError::FindUserError)?
        else {
            return Err(AuthError::UserNotFound);
        };

        self.hsm_store
            .set(user.id, TOTP_REG_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;

        let (_, auth_url) = self
            .for_totp
            .auth_url(user.username.clone(), "TODO_ISSUER".to_string())
            .await
            .map_err(AuthError::TotpError)?;

        self.hsm_store
            .set(user.id, TOTP_REG_STATE, &auth_url)
            .map_err(AuthError::SetHsmStoreError)?;

        Ok(TOTPStartRegistrationOutput { auth_url })
    }
}
