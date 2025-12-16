use crate::{
    domain::{claims::Claims, user::User},
    port::{
        for_auth_tokens::ForAuthTokens,
        for_totp::{ForTotp, TotpError},
        hsm_store::{HSMStore, HSMStoreError},
        user_repository::{UserRepository, UserRepositoryError},
    },
};
use bcrypt::{DEFAULT_COST, hash, verify};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use webauthn_rs::{
    Webauthn,
    prelude::{
        PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
        RegisterPublicKeyCredential, WebauthnError,
    },
};

const WEBAUTHN_REG_STATE: &str = "webauthn/reg/state";
const WEBAUTHN_AUTH_STATE: &str = "webauthn/auth/state";

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid username or password.")]
    InvalidUsernameOrPassword,

    #[error("Failed to create MFA token.")]
    MFATokenCreationFailed,

    #[error("Failed to create refresh token.")]
    RefreshTokenCreationFailed,

    #[error("Failed to create access token.")]
    AccessTokenCreationFailed,

    #[error("Token validation failed.")]
    TokenValidationFailed,

    #[error("User not found.")]
    UserNotFound,

    #[error("WebAuthn registration state not found.")]
    WebAuthnRegistrationNotFound,

    #[error("WebAuthn authentication state not found.")]
    WebAuthnAuthenticationNotFound,

    #[error("Failed to read from HSM store: {0}")]
    GetHsmStoreError(HSMStoreError),

    #[error("Failed to write to HSM store: {0}")]
    SetHsmStoreError(HSMStoreError),

    #[error("Password hashing or verification failed: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),

    #[error("Serialization or deserialization failed: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Failed to retrieve user data: {0}")]
    FindUserError(UserRepositoryError),

    #[error("Failed to persist user data: {0}")]
    SaveUserError(UserRepositoryError),

    #[error("TOTP error: {0}")]
    TotpError(TotpError),

    #[error("WebAuthn error: {0}")]
    WebauthnError(WebauthnError),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResult {
    pub mfa_registration_token: Option<String>,
    pub mfa_verification_token: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

pub struct Auth {
    user_repository: Arc<dyn UserRepository>,
    for_auth_tokens: Arc<dyn ForAuthTokens>,
    for_totp: Arc<dyn ForTotp>,
    hsm_store: Arc<dyn HSMStore>,
    webauthn: Arc<Webauthn>,
}

impl Auth {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        for_auth_tokens: Arc<dyn ForAuthTokens>,
        for_totp: Arc<dyn ForTotp>,
        hsm_store: Arc<dyn HSMStore>,
        webauthn: Arc<Webauthn>,
    ) -> Self {
        Self {
            user_repository,
            for_auth_tokens,
            for_totp,
            hsm_store,
            webauthn,
        }
    }

    pub async fn signup(
        &self,
        name: String,
        username: String,
        password: String,
    ) -> Result<(), AuthError> {
        let cred = User::new(
            username,
            name,
            hash(password, DEFAULT_COST).map_err(AuthError::BcryptError)?,
        );

        self.user_repository
            .save(cred)
            .await
            .map_err(AuthError::SaveUserError)?;
        Ok(())
    }

    pub async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<LoginResult, AuthError> {
        let credential = self
            .user_repository
            .find_username(username.clone())
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        if !verify(password, &credential.password).map_err(AuthError::BcryptError)? {
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
                    sub: username.clone(),
                    exp,
                })
                .await
                .map_err(|_| AuthError::MFATokenCreationFailed)?;

            return Ok(LoginResult {
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
                    sub: username.clone(),
                    exp,
                })
                .await
                .map_err(|_| AuthError::MFATokenCreationFailed)?;

            return Ok(LoginResult {
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

        // return Ok(LoginResult {
        //     mfa_token: None,
        //     access_token: Some(access_token),
        //     refresh_token: Some(refresh_token),
        // });
    }

    pub async fn mfa_totp_setup(&self, mfa_token: String) -> Result<String, AuthError> {
        let claims = self
            .for_auth_tokens
            .validate_token(mfa_token, "mfa_registration".to_string())
            .await
            .map_err(|_| AuthError::TokenValidationFailed)?;

        let Some(mut credential) = self
            .user_repository
            .find_username(claims.sub)
            .await
            .map_err(AuthError::FindUserError)?
        else {
            return Err(AuthError::UserNotFound);
        };

        let (secret, auth_url) = self
            .for_totp
            .auth_url(credential.username.clone(), "TODO_ISSUER".to_string())
            .await
            .map_err(AuthError::TotpError)?;

        credential.otp_secret = Some(secret);
        self.user_repository
            .save(credential)
            .await
            .map_err(AuthError::SaveUserError)?;

        Ok(auth_url)
    }

    pub async fn start_passkey_registration(&self, user_id: Uuid) -> Result<String, AuthError> {
        let Some(user) = self
            .user_repository
            .find_id(user_id)
            .await
            .map_err(AuthError::FindUserError)?
        else {
            return Err(AuthError::UserNotFound);
        };

        self.hsm_store
            .set(user.id, WEBAUTHN_REG_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;

        let credential_ids = user.pass_keys.iter().map(|k| k.cred_id().clone()).collect();

        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(user_id, &user.username, &user.name, Some(credential_ids))
            .expect("Failed to start registration.");

        let json_reg_state = serde_json::to_string(&reg_state).map_err(AuthError::SerdeError)?;
        self.hsm_store
            .set(user.id, WEBAUTHN_REG_STATE, &json_reg_state)
            .map_err(AuthError::SetHsmStoreError)?;

        let ccr = serde_json::to_string(&ccr).map_err(AuthError::SerdeError)?;

        return Ok(ccr);
    }

    pub async fn finish_passkey_registration(
        &self,
        user_id: Uuid,
        req: RegisterPublicKeyCredential,
    ) -> Result<(), AuthError> {
        let reg_state_str = self
            .hsm_store
            .get(user_id, WEBAUTHN_REG_STATE)
            .map_err(AuthError::GetHsmStoreError)?
            .ok_or_else(|| AuthError::WebAuthnRegistrationNotFound)?;

        let reg_state: PasskeyRegistration =
            serde_json::from_str(&reg_state_str).map_err(AuthError::SerdeError)?;
        self.hsm_store
            .set(user_id, WEBAUTHN_REG_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;

        let sk = self
            .webauthn
            .finish_passkey_registration(&req, &reg_state)
            .map_err(AuthError::WebauthnError)?;

        let mut user = self
            .user_repository
            .find_id(user_id)
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        user.pass_keys.push(sk);
        self.user_repository
            .save(user)
            .await
            .map_err(AuthError::SaveUserError)?;

        Ok(())
    }

    pub async fn start_passkey_authentication(&self, user_id: Uuid) -> Result<String, AuthError> {
        self.hsm_store
            .set(user_id, WEBAUTHN_AUTH_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;
        let user = self
            .user_repository
            .find_id(user_id)
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&user.pass_keys)
            .map_err(AuthError::WebauthnError)?;

        let json_auth_state = serde_json::to_string(&auth_state).map_err(AuthError::SerdeError)?;
        self.hsm_store
            .set(user_id, WEBAUTHN_AUTH_STATE, &json_auth_state)
            .map_err(AuthError::SetHsmStoreError)?;

        let rcr = serde_json::to_string(&rcr).map_err(AuthError::SerdeError)?;
        Ok(rcr)
    }

    pub async fn finish_passkey_authentication(
        &self,
        user_id: Uuid,
        auth: PublicKeyCredential,
    ) -> Result<(), AuthError> {
        let auth_state_str = self
            .hsm_store
            .get(user_id, WEBAUTHN_AUTH_STATE)
            .map_err(AuthError::GetHsmStoreError)?
            .ok_or_else(|| AuthError::WebAuthnAuthenticationNotFound)?;

        self.hsm_store
            .set(user_id, WEBAUTHN_AUTH_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;
        let auth_state: PasskeyAuthentication =
            serde_json::from_str(&auth_state_str).map_err(AuthError::SerdeError)?;

        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&auth, &auth_state)
            .map_err(AuthError::WebauthnError)?;

        let mut user = self
            .user_repository
            .find_id(user_id)
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        user.pass_keys.iter_mut().for_each(|k| {
            k.update_credential(&auth_result);
        });
        self.user_repository
            .save(user)
            .await
            .map_err(AuthError::SaveUserError)?;

        Ok(())
    }
}
