use crate::entities::totp::WEBAUTHN_REG_STATE;
use crate::ports::hsm_store::HSMStore;
use crate::ports::user_repository::UserRepository;
use contracts::auth::error::AuthError;
use contracts::auth::passkey::PasskeyFinishRegistrationInput;
use std::sync::Arc;
use webauthn_rs::Webauthn;
use webauthn_rs::prelude::PasskeyRegistration;

pub struct FinishPasskeyRegistrationUseCase {
    user_repository: Arc<dyn UserRepository>,
    webauthn: Arc<Webauthn>,
    hsm_store: Arc<dyn HSMStore>,
}

impl FinishPasskeyRegistrationUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        webauthn: Arc<Webauthn>,
        hsm_store: Arc<dyn HSMStore>,
    ) -> Self {
        Self {
            user_repository,
            webauthn,
            hsm_store,
        }
    }

    pub async fn execute(&self, input: PasskeyFinishRegistrationInput) -> Result<(), AuthError> {
        let reg_state_str = self
            .hsm_store
            .get(input.user_id, WEBAUTHN_REG_STATE)
            .map_err(AuthError::GetHsmStoreError)?
            .ok_or_else(|| AuthError::WebAuthnRegistrationNotFound)?;

        let reg_state: PasskeyRegistration =
            serde_json::from_str(&reg_state_str).map_err(AuthError::SerdeError)?;
        self.hsm_store
            .set(input.user_id, WEBAUTHN_REG_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;

        let sk = self
            .webauthn
            .finish_passkey_registration(&input.register_public_key_credential, &reg_state)
            .map_err(AuthError::WebauthnError)?;

        let mut user = self
            .user_repository
            .find_id(input.user_id)
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
}
