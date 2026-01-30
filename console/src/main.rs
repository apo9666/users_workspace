use auth::AuthComponent;
use contracts::auth::login::LoginInput;
use contracts::auth::mfa::MfaRegistrationInput;
use contracts::auth::signup::SignupInput;
use contracts::auth::totp::TOTPFinishRegistrationInput;
use contracts::auth::{Component, totp::TOTPStartRegistrationInput};
use env_logger::{Builder, Target};
use totp_rs::Secret;

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
        .get_mfa_registration(MfaRegistrationInput {
            access_token: result.access_token.unwrap(),
        })
        .await
        .unwrap();

    let mfa_registration = result.mfa_registration;
    let result = auth
        .start_totp_registration(TOTPStartRegistrationInput {
            mfa_token: mfa_registration.clone(),
        })
        .await
        .unwrap();
    println!("OTP Secret: {}", result.auth_url);

    // extract the base32 secret from the otpauth URL returned by start_totp_registration
    let auth_url = result.auth_url.clone();
    let secret = auth_url
        .split("secret=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .expect("no secret in auth_url")
        .to_string();

    // generate the current TOTP using totp-rs
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        6,
        0,
        30,
        Secret::Encoded(secret).to_bytes().unwrap(),
    )
    .expect("failed to create TOTP");
    let code = totp.generate_current().expect("failed to generate TOTP");

    println!("Generated TOTP code: {}", &code);

    let result = auth
        .finish_totp_registration(TOTPFinishRegistrationInput {
            mfa_token: mfa_registration,
            code,
        })
        .await
        .unwrap();

    println!("MFA Token: {}", result.access_token);
}
