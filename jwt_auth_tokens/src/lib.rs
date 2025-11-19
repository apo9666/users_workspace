use application::{
    domain::claims::Claims,
    port::for_auth_tokens::{AuthTokenError, ForAuthTokens},
};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation, decode, encode};
use log::error;

mod read_key;
mod read_public;

pub struct JwtAuthTokens {}

#[async_trait]
impl ForAuthTokens for JwtAuthTokens {
    async fn create_token(&self, claims: Claims) -> Result<String, AuthTokenError> {
        let (name, data) = read_key::get_first_key_cached("./ed25519").await?;

        let encoding_key = EncodingKey::from_ed_pem(&data).map_err(|err| {
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

        let header = jsonwebtoken::Header {
            alg: Algorithm::EdDSA,
            kid: Some(name),
            ..Default::default()
        };
        let token = encode(&header, &claims, &encoding_key).map_err(|err| {
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
        let kid = {
            let header = jsonwebtoken::decode_header(token).map_err(|err| match *err.kind() {
                jsonwebtoken::errors::ErrorKind::InvalidToken => AuthTokenError::InvalidToken,
                _ => {
                    error!("An error occurred while decoding the token header: {}", err);
                    AuthTokenError::InvalidToken
                }
            })?;
            header.kid.ok_or_else(|| {
                error!("No 'kid' found in token header.");
                AuthTokenError::InvalidToken
            })?
        };
        let jwks = read_public::build_jwks_from_dir("./ed25519").await?;
        let jwk = jwks.find(&kid).ok_or_else(|| {
            error!("No matching JWK found for kid: {}", kid);
            AuthTokenError::InvalidToken
        })?;
        let decoding_key = DecodingKey::from_jwk(jwk).map_err(|err| {
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
