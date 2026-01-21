use auth::AuthComponent;
use contracts::auth::login::LoginInput;
use contracts::auth::signup::SignupInput;
use contracts::auth::{Component, totp::TOTPStartRegistrationInput};
use env_logger::{Builder, Target};

#[tokio::main]
async fn main() {
    Builder::new().target(Target::Stdout).init();

    let auth = AuthComponent::new();

    auth.signup(SignupInput {
        name: "User 1".to_string(),
        username: "user1".to_string(),
        password: "password123".to_string(),
    })
    .await
    .unwrap();

    let result = auth
        .login(LoginInput {
            username: "user1".to_string(),
            password: "password123".to_string(),
        })
        .await
        .unwrap();

    let result = auth
        .start_totp_registration(TOTPStartRegistrationInput {
            mfa_token: result.mfa_registration_token.unwrap(),
        })
        .await
        .unwrap();
    println!("OTP Secret: {}", result.auth_url);

    let result = auth
        .login(LoginInput {
            username: "user1".to_string(),
            password: "password123".to_string(),
        })
        .await
        .unwrap();

    println!("MFA Token: {}", result.mfa_registration_token.unwrap());
}
