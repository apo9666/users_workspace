use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum TotpError {
    #[error("Failed to verify TOTP code.")]
    VerificationFailed,

    #[error("Failed to generate authentication URL.")]
    AuthUrlGenerationFailed,
}

#[async_trait]
pub trait ForTotp {
    async fn verify(&self, secret: String, token: String) -> Result<bool, TotpError>;
    async fn auth_url(&self, issuer: String, email: String) -> Result<(String, String), TotpError>;
}
