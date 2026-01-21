use async_trait::async_trait;
use contracts::auth::error::TotpError;

#[async_trait]
pub trait ForTotp: Send + Sync {
    async fn verify(&self, secret: String, token: String) -> Result<bool, TotpError>;
    async fn auth_url(&self, issuer: String, email: String) -> Result<(String, String), TotpError>;
}
