use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize)]
pub struct TotpSetupResponse {
    pub qr_code_url: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct TotpVerifyRequest {
    #[validate(length(
        min = 6,
        max = 6,
        message = "O código deve ter 6 dígitos",
        code = "name_too_short"
    ))]
    pub code: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TotpVerifyResponse {
    pub access_token: String,
    pub refresh_token: String,
}
