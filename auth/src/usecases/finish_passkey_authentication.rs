use crate::entities::totp::WEBAUTHN_AUTH_STATE;
use crate::ports::hsm_store::HSMStore;
use crate::ports::user_repository::UserRepository;
use contracts::auth::error::AuthError;
use contracts::auth::passkey::PasskeyFinishAuthenticationInput;
use std::sync::Arc;
use webauthn_rs::Webauthn;
use webauthn_rs::prelude::PasskeyAuthentication;

pub struct FinishPasskeyAuthenticationUseCase {
    user_repository: Arc<dyn UserRepository>,
    hsm_store: Arc<dyn HSMStore>,
    webauthn: Arc<Webauthn>,
}

impl FinishPasskeyAuthenticationUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        hsm_store: Arc<dyn HSMStore>,
        webauthn: Arc<Webauthn>,
    ) -> Self {
        Self {
            user_repository,
            hsm_store,
            webauthn,
        }
    }

    pub async fn execute(&self, input: PasskeyFinishAuthenticationInput) -> Result<(), AuthError> {
        let auth_state_str = self
            .hsm_store
            .get(input.user_id, WEBAUTHN_AUTH_STATE)
            .map_err(AuthError::GetHsmStoreError)?
            .ok_or_else(|| AuthError::WebAuthnAuthenticationNotFound)?;

        self.hsm_store
            .set(input.user_id, WEBAUTHN_AUTH_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;
        let auth_state: PasskeyAuthentication =
            serde_json::from_str(&auth_state_str).map_err(AuthError::SerdeError)?;

        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&input.public_key_credential, &auth_state)
            .map_err(AuthError::WebauthnError)?;

        let mut user = self
            .user_repository
            .find_id(input.user_id)
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
