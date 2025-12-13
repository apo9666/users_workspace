use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SignupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}
