use application::{
    domain::claims::Claims,
    port::for_auth_tokens::{AuthTokenError, ForAuthTokens},
};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation, decode, encode};
use log::error;

pub struct JwtAuthTokens {}

#[async_trait]
impl ForAuthTokens for JwtAuthTokens {
    async fn create_token(&self, claims: Claims) -> Result<String, AuthTokenError> {
        let key_pem = include_bytes!("./ed25519_key.pem");
        let encoding_key = EncodingKey::from_ed_pem(key_pem).map_err(|err| {
            match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidKeyFormat => {
                    error!("Invalid key format provided for token creation.");
                }
                _ => {
                    error!("An error occurred while creating the encoding key: {}", err);
                }
            }
            AuthTokenError::TokenCreationFailure
        })?;

        let token = encode(
            &jsonwebtoken::Header::new(Algorithm::EdDSA),
            &claims,
            &encoding_key,
        )
        .map_err(|err| {
            match *err.kind() {
                _ => {
                    error!("An error occurred while encoding the token: {}", err);
                }
            }
            AuthTokenError::TokenCreationFailure
        })?;

        Ok(token)
    }

    async fn validate_token(&self, token: &str) -> Result<Claims, AuthTokenError> {
        let pub_pem = include_bytes!("./ed25519_public.pem");
        let decoding_key = DecodingKey::from_ed_pem(pub_pem).map_err(|err| {
            match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidKeyFormat => {
                    error!("Invalid key format provided for token validation.");
                }
                _ => {
                    error!("An error occurred while creating the decoding key: {}", err);
                }
            }
            AuthTokenError::TokenCreationFailure
        })?;

        let validation = Validation::new(Algorithm::EdDSA);
        let claims =
            decode::<Claims>(&token, &decoding_key, &validation).map_err(|err| {
                match *err.kind() {
                    jsonwebtoken::errors::ErrorKind::InvalidToken => AuthTokenError::InvalidToken,
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        AuthTokenError::TokenExpired
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        AuthTokenError::InvalidSignature
                    }
                    _ => {
                        error!("An error occurred while decoding the token: {}", err);
                        AuthTokenError::InvalidToken
                    }
                }
            })?;

        Ok(claims.claims)
    }
}
