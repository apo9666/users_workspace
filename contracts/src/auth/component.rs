use crate::auth::{error::AuthError, login, passkey, signup, totp};

#[async_trait::async_trait]
pub trait Component {
    async fn login(&self, input: login::LoginInput) -> Result<login::LoginOutput, AuthError>;

    async fn signup(&self, input: signup::SignupInput) -> Result<signup::SignupOutput, AuthError>;

    async fn start_totp_registration(
        &self,
        input: totp::TOTPStartRegistrationInput,
    ) -> Result<totp::TOTPStartRegistrationOutput, AuthError>;

    async fn finish_totp_registration(
        &self,
        input: totp::TOTPFinishRegistrationInput,
    ) -> Result<totp::TOTPFinishRegistrationOutput, AuthError>;

    async fn start_passkey_registration(
        &self,
        input: passkey::PasskeyStartRegistrationInput,
    ) -> Result<passkey::PasskeyStartRegistrationOutput, AuthError>;

    async fn finish_passkey_registration(
        &self,
        input: passkey::PasskeyFinishRegistrationInput,
    ) -> Result<(), AuthError>;

    async fn start_passkey_authentication(
        &self,
        input: passkey::PasskeyStartAuthenticationInput,
    ) -> Result<passkey::PasskeyStartAuthenticationOutput, AuthError>;

    async fn finish_passkey_authentication(
        &self,
        input: passkey::PasskeyFinishAuthenticationInput,
    ) -> Result<(), AuthError>;
}
