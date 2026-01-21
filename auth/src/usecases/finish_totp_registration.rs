use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use contracts::auth::{
    error::AuthError,
    totp::{TOTPFinishRegistrationInput, TOTPFinishRegistrationOutput},
};
use webauthn_rs::prelude::Url;

use crate::{
    entities::{claims::Claims, totp::TOTP_REG_STATE},
    ports::{
        for_auth_tokens::ForAuthTokens, for_totp::ForTotp, hsm_store::HSMStore,
        user_repository::UserRepository,
    },
};

pub struct FinishTOTPRegistrationUseCase {
    user_repository: Arc<dyn UserRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
    for_totp: Arc<dyn ForTotp>,
    hsm_store: Arc<dyn HSMStore>,
}

impl FinishTOTPRegistrationUseCase {
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
        input: TOTPFinishRegistrationInput,
    ) -> Result<TOTPFinishRegistrationOutput, AuthError> {
        let claims = self
            .for_auth_tokens
            .validate_token(input.mfa_token, "mfa_registration".to_string())
            .await
            .map_err(|_| AuthError::TokenValidationFailed)?;

        let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AuthError::UserNotFound)?;

        let Some(mut user) = self
            .user_repository
            .find_id(user_id)
            .await
            .map_err(AuthError::FindUserError)?
        else {
            return Err(AuthError::UserNotFound);
        };

        let reg_state_str = self
            .hsm_store
            .get(user.id, TOTP_REG_STATE)
            .map_err(AuthError::GetHsmStoreError)?
            .ok_or_else(|| AuthError::TotpRegistrationNotFound)?;

        self.hsm_store
            .set(user.id, TOTP_REG_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;

        let secret = Url::parse(&reg_state_str)
            .ok()
            .and_then(|u| {
                u.query_pairs()
                    .find(|(key, _)| key == "secret")
                    .map(|(_, val)| val.into_owned())
            })
            .ok_or_else(|| AuthError::TotpRegistrationNotFound)?;

        let result = self
            .for_totp
            .verify(secret.clone(), input.code)
            .await
            .map_err(AuthError::TotpError)?;

        if !result {
            return Err(AuthError::MFATokenCreationFailed);
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
                sub: user.id.to_string(),
                exp,
            })
            .await
            .map_err(|_| AuthError::RefreshTokenCreationFailed)?;

        let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
        let access_token = self
            .for_auth_tokens
            .create_token(Claims {
                token_type: "access".to_string(),
                sub: user.id.to_string(),
                exp,
            })
            .await
            .map_err(|_| AuthError::AccessTokenCreationFailed)?;

        user.otp_secret = Some(secret);
        self.user_repository
            .save(user)
            .await
            .map_err(AuthError::SaveUserError)?;

        Ok(TOTPFinishRegistrationOutput {
            access_token,
            refresh_token,
        })
    }
}
