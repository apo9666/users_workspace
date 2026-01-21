#[derive(Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginOutput {
    pub mfa_registration_token: Option<String>,
    pub mfa_verification_token: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}
