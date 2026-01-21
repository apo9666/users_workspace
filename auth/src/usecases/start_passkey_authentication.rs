use crate::entities::totp::WEBAUTHN_AUTH_STATE;
use crate::ports::hsm_store::HSMStore;
use crate::ports::user_repository::UserRepository;
use contracts::auth::error::AuthError;
use contracts::auth::passkey::{PasskeyStartAuthenticationInput, PasskeyStartAuthenticationOutput};
use std::sync::Arc;
use webauthn_rs::Webauthn;

pub struct StartPasskeyAuthenticationUseCase {
    user_repository: Arc<dyn UserRepository>,
    hsm_store: Arc<dyn HSMStore>,
    webauthn: Arc<Webauthn>,
}
impl StartPasskeyAuthenticationUseCase {
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

    pub async fn execute(
        &self,
        input: PasskeyStartAuthenticationInput,
    ) -> Result<PasskeyStartAuthenticationOutput, AuthError> {
        self.hsm_store
            .set(input.user_id, WEBAUTHN_AUTH_STATE, "")
            .map_err(AuthError::SetHsmStoreError)?;
        let user = self
            .user_repository
            .find_id(input.user_id)
            .await
            .map_err(AuthError::FindUserError)?
            .ok_or_else(|| AuthError::UserNotFound)?;

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&user.pass_keys)
            .map_err(AuthError::WebauthnError)?;

        let json_auth_state = serde_json::to_string(&auth_state).map_err(AuthError::SerdeError)?;
        self.hsm_store
            .set(input.user_id, WEBAUTHN_AUTH_STATE, &json_auth_state)
            .map_err(AuthError::SetHsmStoreError)?;

        let rcr = serde_json::to_string(&rcr).map_err(AuthError::SerdeError)?;
        Ok(PasskeyStartAuthenticationOutput { challenge: rcr })
    }
}
