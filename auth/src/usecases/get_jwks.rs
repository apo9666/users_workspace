use std::sync::Arc;

use contracts::auth::error::AuthError;

use crate::ports::for_auth_tokens::ForAuthTokens;

pub struct GetJwksUseCase {
    for_auth_tokens: Arc<dyn ForAuthTokens>,
}

impl GetJwksUseCase {
    pub fn new(for_auth_tokens: Arc<dyn ForAuthTokens>) -> Self {
        Self { for_auth_tokens }
    }

    pub async fn execute(&self) -> Result<String, AuthError> {
        self.for_auth_tokens
            .get_jwks()
            .await
            .map_err(|_| AuthError::JwksFetchFailed)
    }
}
