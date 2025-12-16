use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TotpSetupResponse {
    pub qr_code_url: String,
}
