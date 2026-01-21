use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub username: String,
    pub password: String,
    pub otp_secret: Option<String>,
    pub pass_keys: Vec<Passkey>,
}

impl User {
    pub fn new(username: &str, name: &str, password: &str) -> Self {
        User {
            id: Uuid::new_v4(),
            username: username.to_string(),
            name: name.to_string(),
            password: password.to_string(),
            pass_keys: Vec::new(),
            otp_secret: None,
        }
    }
}
