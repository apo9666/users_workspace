use uuid::Uuid;
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential};

#[derive(Debug)]
pub struct PasskeyStartRegistrationInput {
    pub user_id: Uuid,
}

#[derive(Debug)]
pub struct PasskeyStartRegistrationOutput {
    pub challenge: String,
}

#[derive(Debug)]
pub struct PasskeyFinishRegistrationInput {
    pub user_id: Uuid,
    pub register_public_key_credential: RegisterPublicKeyCredential,
}

#[derive(Debug)]
pub struct PasskeyStartAuthenticationInput {
    pub user_id: Uuid,
    pub response: String,
}

#[derive(Debug)]
pub struct PasskeyStartAuthenticationOutput {
    pub challenge: String,
}

#[derive(Debug)]
pub struct PasskeyFinishAuthenticationInput {
    pub user_id: Uuid,
    pub public_key_credential: PublicKeyCredential,
}
