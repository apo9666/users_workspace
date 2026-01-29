#[derive(Debug)]
pub struct MfaRegistrationInput {
    pub access_token: String,
}

#[derive(Debug)]
pub struct MfaRegistrationOutput {
    pub mfa_registration: String,
    pub allowed_methods: Vec<String>,
    pub expires_in: usize,
}
