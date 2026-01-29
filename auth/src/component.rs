use crate::{
    infra::{
        jwt_auth_tokens::JwtAuthTokens,
        memory::{hsm_store::MemoryHsmStore, user_repository},
        totp::Totp,
    },
    ports::user_repository::UserRepository,
    usecases::{
        finish_passkey_authentication::FinishPasskeyAuthenticationUseCase,
        finish_passkey_registration::FinishPasskeyRegistrationUseCase,
        finish_totp_registration::FinishTOTPRegistrationUseCase, get_jwks::GetJwksUseCase,
        get_mfa_registration::GetMfaRegistrationUseCase, login::LoginUseCase,
        signup::SignupUseCase,
        start_passkey_authentication::StartPasskeyAuthenticationUseCase,
        start_passkey_registration::StartPasskeyRegistrationUseCase,
        start_totp_registration::StartTOTPRegistrationUseCase,
    },
};
use contracts::{
        self,
        auth::{
            self,
            error::AuthError,
            login::{LoginInput, LoginOutput},
            mfa::{MfaRegistrationInput, MfaRegistrationOutput},
            passkey::{
                PasskeyFinishAuthenticationInput, PasskeyFinishRegistrationInput,
                PasskeyStartAuthenticationInput, PasskeyStartAuthenticationOutput,
                PasskeyStartRegistrationInput, PasskeyStartRegistrationOutput,
        },
        signup::{SignupInput, SignupOutput},
        totp::{
            TOTPFinishRegistrationInput, TOTPFinishRegistrationOutput, TOTPStartRegistrationInput,
            TOTPStartRegistrationOutput,
        },
    },
};
use std::sync::Arc;
use webauthn_rs::{WebauthnBuilder, prelude::Url};

pub struct AuthComponent {
    login_usecase: Arc<LoginUseCase>,
    signup_usecase: Arc<SignupUseCase>,
    start_totp_registration_usecase: Arc<StartTOTPRegistrationUseCase>,
    finish_totp_registration_usecase: Arc<FinishTOTPRegistrationUseCase>,
    start_passkey_registration_usecase: Arc<StartPasskeyRegistrationUseCase>,
    finish_passkey_registration_usecase: Arc<FinishPasskeyRegistrationUseCase>,
    start_passkey_authentication_usecase: Arc<StartPasskeyAuthenticationUseCase>,
    finish_passkey_authentication_usecase: Arc<FinishPasskeyAuthenticationUseCase>,
    get_mfa_registration_usecase: Arc<GetMfaRegistrationUseCase>,
    get_jwks_usecase: Arc<GetJwksUseCase>,
}

impl AuthComponent {
    pub fn new() -> impl auth::Component {
        let user_repository: Arc<dyn UserRepository> =
            Arc::new(user_repository::MemoryUserRepository::new());
        let jwt_auth = Arc::new(JwtAuthTokens {});
        let totp = Arc::new(Totp {});
        let hsm_store = Arc::new(MemoryHsmStore::new());
        let webauthn = Arc::new(
            WebauthnBuilder::new("localhost", &Url::parse("http://localhost:3000").unwrap())
                .unwrap()
                .build()
                .unwrap(),
        );

        AuthComponent {
            login_usecase: Arc::new(LoginUseCase::new(user_repository.clone(), jwt_auth.clone())),
            signup_usecase: Arc::new(SignupUseCase::new(user_repository.clone())),
            start_totp_registration_usecase: Arc::new(StartTOTPRegistrationUseCase::new(
                user_repository.clone(),
                jwt_auth.clone(),
                totp.clone(),
                hsm_store.clone(),
            )),
            finish_totp_registration_usecase: Arc::new(FinishTOTPRegistrationUseCase::new(
                user_repository.clone(),
                jwt_auth.clone(),
                totp.clone(),
                hsm_store.clone(),
            )),
            start_passkey_registration_usecase: Arc::new(StartPasskeyRegistrationUseCase::new(
                user_repository.clone(),
                webauthn.clone(),
                hsm_store.clone(),
            )),
            finish_passkey_registration_usecase: Arc::new(FinishPasskeyRegistrationUseCase::new(
                user_repository.clone(),
                webauthn.clone(),
                hsm_store.clone(),
            )),
            start_passkey_authentication_usecase: Arc::new(StartPasskeyAuthenticationUseCase::new(
                user_repository.clone(),
                hsm_store.clone(),
                webauthn.clone(),
            )),
            finish_passkey_authentication_usecase: Arc::new(
                FinishPasskeyAuthenticationUseCase::new(
                    user_repository.clone(),
                    hsm_store,
                    webauthn,
                ),
            ),
            get_mfa_registration_usecase: Arc::new(GetMfaRegistrationUseCase::new(
                user_repository.clone(),
                jwt_auth.clone(),
            )),
            get_jwks_usecase: Arc::new(GetJwksUseCase::new(jwt_auth)),
        }
    }
}

#[async_trait::async_trait]
impl auth::Component for AuthComponent {
    async fn login(&self, input: LoginInput) -> Result<LoginOutput, AuthError> {
        self.login_usecase.execute(input).await
    }

    async fn signup(&self, input: SignupInput) -> Result<SignupOutput, AuthError> {
        self.signup_usecase.execute(input).await
    }

    async fn start_totp_registration(
        &self,
        input: TOTPStartRegistrationInput,
    ) -> Result<TOTPStartRegistrationOutput, AuthError> {
        self.start_totp_registration_usecase.execute(input).await
    }

    async fn finish_totp_registration(
        &self,
        input: TOTPFinishRegistrationInput,
    ) -> Result<TOTPFinishRegistrationOutput, AuthError> {
        self.finish_totp_registration_usecase.execute(input).await
    }

    async fn start_passkey_registration(
        &self,
        input: PasskeyStartRegistrationInput,
    ) -> Result<PasskeyStartRegistrationOutput, AuthError> {
        self.start_passkey_registration_usecase.execute(input).await
    }

    async fn finish_passkey_registration(
        &self,
        input: PasskeyFinishRegistrationInput,
    ) -> Result<(), AuthError> {
        self.finish_passkey_registration_usecase
            .execute(input)
            .await
    }

    async fn start_passkey_authentication(
        &self,
        input: PasskeyStartAuthenticationInput,
    ) -> Result<PasskeyStartAuthenticationOutput, AuthError> {
        self.start_passkey_authentication_usecase
            .execute(input)
            .await
    }

    async fn finish_passkey_authentication(
        &self,
        input: PasskeyFinishAuthenticationInput,
    ) -> Result<(), AuthError> {
        self.finish_passkey_authentication_usecase
            .execute(input)
            .await
    }

    async fn get_mfa_registration(
        &self,
        input: MfaRegistrationInput,
    ) -> Result<MfaRegistrationOutput, AuthError> {
        self.get_mfa_registration_usecase.execute(input).await
    }

    async fn get_jwks(&self) -> Result<String, AuthError> {
        self.get_jwks_usecase.execute().await
    }
}
