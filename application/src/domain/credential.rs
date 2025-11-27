#[derive(Clone, Debug)]
pub struct Credential {
    pub username: String,
    pub password: String,
    pub otp_secret: Option<String>,
}
