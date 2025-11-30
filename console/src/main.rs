use application::auth::Auth;
use env_logger::{Builder, Target};
use jwt_auth_tokens::JwtAuthTokens;
use memory::repository::credential::MemoryCredentialRepository;
use std::sync::Arc;
use totp::Totp;

#[tokio::main]
async fn main() {
    Builder::new().target(Target::Stdout).init();

    let credential_repository = Arc::new(MemoryCredentialRepository::new());
    let jwt_auth = Arc::new(JwtAuthTokens {});
    let totp = Arc::new(Totp {});

    let auth = Auth::new(
        credential_repository.clone(),
        jwt_auth.clone(),
        totp.clone(),
    );

    auth.signup("user1".to_string(), "password123".to_string())
        .await
        .unwrap();

    let result = auth
        .login("user1".to_string(), "password123".to_string())
        .await
        .unwrap();

    let result = auth
        .mfa_totp_setup(result.access_token.unwrap())
        .await
        .unwrap();
    println!("OTP Secret: {}", result);

    let result = auth
        .login("user1".to_string(), "password123".to_string())
        .await
        .unwrap();

    println!("MFA Token: {}", result.mfa_token.unwrap());
}
