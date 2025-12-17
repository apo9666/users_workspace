use application::auth::Auth;
use env_logger::{Builder, Target};
use jwt_auth_tokens::JwtAuthTokens;
use memory::repository::user::MemoryUserRepository;
use std::sync::Arc;
use totp::Totp;
use webauthn_rs::{WebauthnBuilder, prelude::Url};

#[tokio::main]
async fn main() {
    Builder::new().target(Target::Stdout).init();

    let credential_repository = Arc::new(MemoryUserRepository::new());
    let jwt_auth = Arc::new(JwtAuthTokens {});
    let totp = Arc::new(Totp {});
    let hsm_store = Arc::new(memory::hsm_store::MemoryHsmStore::new());
    let webauthn = Arc::new(
        WebauthnBuilder::new("localhost", &Url::parse("http://localhost:3000").unwrap())
            .unwrap()
            .build()
            .unwrap(),
    );

    let auth = Auth::new(
        credential_repository.clone(),
        jwt_auth.clone(),
        totp.clone(),
        hsm_store.clone(),
        webauthn.clone(),
    );

    auth.signup(
        "User 1".to_string(),
        "user1".to_string(),
        "password123".to_string(),
    )
    .await
    .unwrap();

    let result = auth
        .login("user1".to_string(), "password123".to_string())
        .await
        .unwrap();

    let result = auth
        .start_totp_registration(result.access_token.unwrap())
        .await
        .unwrap();
    println!("OTP Secret: {}", result);

    let result = auth
        .login("user1".to_string(), "password123".to_string())
        .await
        .unwrap();

    println!("MFA Token: {}", result.mfa_registration_token.unwrap());
}
