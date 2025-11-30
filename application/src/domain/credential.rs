use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Credential {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub otp_secret: Option<String>,
}
