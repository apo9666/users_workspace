use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    entities::claims::Claims,
    ports::{for_auth_tokens::ForAuthTokens, user_repository::UserRepository},
};
use bcrypt::verify;
use contracts::auth::{
    error::AuthError,
    login::{LoginInput, LoginOutput},
};

pub struct LoginUseCase {
    user_repository: Arc<dyn UserRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
}

impl LoginUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
    ) -> Self {
        Self {
            user_repository,
            for_auth_tokens,
        }
    }

    pub async fn execute(&self, input: LoginInput) -> Result<LoginOutput, AuthError> {
        let credential = self
            .user_repository
            .find_username(input.username.to_string())
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        if !verify(&input.password, &credential.password).map_err(AuthError::BcryptError)? {
            return Err(AuthError::InvalidUsernameOrPassword);
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
                    token_type: "mfa_verification".to_string(),
                    sub: credential.id.to_string(),
                    exp,
                })
                .await
                .map_err(|_| AuthError::MFATokenCreationFailed)?;

            return Ok(LoginOutput {
                mfa_registration_token: None,
                mfa_verification_token: Some(mfa_token),
                access_token: None,
                refresh_token: None,
            });
        } else {
            let exp = (since_the_epoch.as_secs() + 3600) as usize; // 1h from now
            let mfa_token = self
                .for_auth_tokens
                .create_token(Claims {
                    token_type: "mfa_registration".to_string(),
                    sub: credential.id.to_string(),
                    exp,
                })
                .await
                .map_err(|_| AuthError::MFATokenCreationFailed)?;

            return Ok(LoginOutput {
                mfa_registration_token: Some(mfa_token),
                mfa_verification_token: None,
                access_token: None,
                refresh_token: None,
            });
        }

        // let exp = (since_the_epoch.as_secs() + 604800) as usize; // 7 days from now
        // let refresh_token = self
        //     .for_auth_tokens
        //     .create_token(Claims {
        //         token_type: "refresh".to_string(),
        //         sub: username.clone(),
        //         exp,
        //     })
        //     .await
        //     .map_err(|_| AuthError::RefreshTokenCreationFailed)?;

        // let exp = (since_the_epoch.as_secs() + 600) as usize; // 10 minutes from now
        // let access_token = self
        //     .for_auth_tokens
        //     .create_token(Claims {
        //         token_type: "access".to_string(),
        //         sub: username.clone(),
        //         exp,
        //     })
        //     .await
        //     .map_err(|_| AuthError::AccessTokenCreationFailed)?;

        // return Ok(LoginOutput {
        //     mfa_token: None,
        //     access_token: Some(access_token),
        //     refresh_token: Some(refresh_token),
        // });
    }
}
