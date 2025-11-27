use application::port::for_totp::{ForTotp, TotpError};
use async_trait::async_trait;
use log::warn;
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

pub struct Totp {}

#[async_trait]
impl ForTotp for Totp {
    async fn verify(&self, secret: String, token: String) -> Result<bool, TotpError> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret).to_bytes().unwrap(),
        )
        .unwrap();

        let result = totp.check_current(&token).map_err(|e| {
            warn!("Error verifying TOTP token: {}", e);
            TotpError::VerificationFailed
        })?;

        Ok(result)
    }

    async fn auth_url(&self, issuer: String, email: String) -> Result<(String, String), TotpError> {
        let mut rng = rand::rng();
        let data_byte: [u8; 21] = rng.random();
        let base32_string =
            base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &data_byte);

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(base32_string).to_bytes().unwrap(),
        )
        .unwrap();

        let otp_base32 = totp.get_secret_base32();
        let otp_auth_url =
            format!("otpauth://totp/{issuer}:{email}?secret={otp_base32}&issuer={issuer}");

        Ok((otp_base32, otp_auth_url))
    }
}
