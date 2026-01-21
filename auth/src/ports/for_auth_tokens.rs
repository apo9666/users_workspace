use async_trait::async_trait;
use contracts::auth::error::AuthTokenError;

use crate::entities::claims::Claims;

#[async_trait]
pub trait ForAuthTokens: Send + Sync {
    async fn create_token(&self, claims: Claims) -> Result<String, AuthTokenError>;
    async fn validate_token(
        &self,
        token: String,
        token_type: String,
    ) -> Result<Claims, AuthTokenError>;
    async fn get_jwks(&self) -> Result<String, AuthTokenError>;
}
