#[derive(Debug)]
pub struct TOTPStartRegistrationInput {
    pub mfa_token: String,
}

#[derive(Debug)]
pub struct TOTPStartRegistrationOutput {
    pub auth_url: String,
}

#[derive(Debug)]
pub struct TOTPFinishRegistrationInput {
    pub code: String,
    pub mfa_token: String,
}

#[derive(Debug)]
pub struct TOTPFinishRegistrationOutput {
    pub access_token: String,
    pub refresh_token: String,
}
