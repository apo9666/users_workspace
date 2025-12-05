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
    pub fn new(username: String, name: String, password: String) -> Self {
        User {
            id: Uuid::new_v4(),
            username,
            name,
            password,
            pass_keys: Vec::new(),
            otp_secret: None,
        }
    }
}
