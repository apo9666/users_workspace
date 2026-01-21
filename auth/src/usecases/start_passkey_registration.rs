use crate::entities::totp::WEBAUTHN_REG_STATE;
use crate::ports::hsm_store::HSMStore;
use crate::ports::user_repository::UserRepository;
use contracts::auth::error::AuthError;
use contracts::auth::passkey::{PasskeyStartRegistrationInput, PasskeyStartRegistrationOutput};
use std::sync::Arc;
use webauthn_rs::Webauthn;

pub struct StartPasskeyRegistrationUseCase {
    user_repository: Arc<dyn UserRepository>,
    webauthn: Arc<Webauthn>,
    hsm_store: Arc<dyn HSMStore>,
}

impl StartPasskeyRegistrationUseCase {
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

    pub async fn execute(
        &self,
        input: PasskeyStartRegistrationInput,
    ) -> Result<PasskeyStartRegistrationOutput, AuthError> {
        let Some(user) = self
            .user_repository
            .find_id(input.user_id)
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
            .start_passkey_registration(
                input.user_id,
                &user.username,
                &user.name,
                Some(credential_ids),
            )
            .expect("Failed to start registration.");

        let json_reg_state = serde_json::to_string(&reg_state).map_err(AuthError::SerdeError)?;
        self.hsm_store
            .set(user.id, WEBAUTHN_REG_STATE, &json_reg_state)
            .map_err(AuthError::SetHsmStoreError)?;

        let ccr = serde_json::to_string(&ccr).map_err(AuthError::SerdeError)?;

        return Ok(PasskeyStartRegistrationOutput { challenge: ccr });
    }
}
