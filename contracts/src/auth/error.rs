use std::{error::Error, fmt};

use webauthn_rs::prelude::WebauthnError;

#[derive(Debug)]
pub enum HSMStoreError {
    StorageError(String),
}

impl std::fmt::Display for HSMStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HSMStoreError::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for HSMStoreError {}

#[derive(Debug, thiserror::Error)]
pub enum TotpError {
    #[error("Failed to verify TOTP code.")]
    VerificationFailed,

    #[error("Failed to generate authentication URL.")]
    AuthUrlGenerationFailed,
}

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

#[derive(Debug)]
pub enum UserRepositoryError {
    ConnectionError(String),
}

impl fmt::Display for UserRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRepositoryError::ConnectionError(msg) => {
                write!(f, "Connection error: {}", msg)
            }
        }
    }
}

impl Error for UserRepositoryError {}

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

    #[error("TOTP registration state not found.")]
    TotpRegistrationNotFound,

    #[error("WebAuthn error: {0}")]
    WebauthnError(WebauthnError),
}

pub enum InternalAuthError {
    FindUserError(UserRepositoryError),
    SetHsmStoreError(HSMStoreError),
    WebauthnError(WebauthnError),
}
