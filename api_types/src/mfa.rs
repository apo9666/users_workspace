use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MfaRegistrationResponse {
    pub mfa_registration: String,
    pub allowed_methods: Vec<String>,
    pub expires_in: usize,
}
