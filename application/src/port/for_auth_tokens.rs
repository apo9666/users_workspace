use async_trait::async_trait;

use crate::domain::claims::Claims;

#[derive(Debug, thiserror::Error)]
pub enum AuthTokenError {
    #[error("Failed to create token due to an internal serialization or signing error.")]
    TokenCreationFailure,

    #[error("The token provided is malformed or invalid.")]
    InvalidToken,

    #[error("The token has expired.")]
    TokenExpired,

    #[error("The token signature is invalid.")]
    InvalidSignature,

    #[error("Failed to fetch JWKs for token validation.")]
    JwksFetchError,
}

#[async_trait]
pub trait ForAuthTokens {
    async fn create_token(&self, claims: Claims) -> Result<String, AuthTokenError>;
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthTokenError>;
    async fn get_jwks(&self) -> Result<String, AuthTokenError>;
}
