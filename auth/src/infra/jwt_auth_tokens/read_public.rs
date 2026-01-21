use base64ct::{Base64, Base64UrlUnpadded, Encoding};
use contracts::auth::error::AuthTokenError;
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, EllipticCurve, Jwk, JwkSet, OctetKeyPairParameters,
    OctetKeyPairType,
};
use log::error;
use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::{fs, sync::RwLock, time::Instant};

const CACHE_TTL_SECS: u64 = 10 * 60; // 10 minutes

#[derive(Debug)]
struct CacheEntry {
    jwks: JwkSet,
    expires_at: Instant,
}

static CACHE: Lazy<RwLock<Option<CacheEntry>>> = Lazy::new(|| RwLock::new(None));

pub async fn build_jwks_from_dir(dir: &str) -> Result<JwkSet, AuthTokenError> {
    {
        let read = CACHE.read().await;
        if let Some(cached) = read.as_ref() {
            if Instant::now() < cached.expires_at {
                return Ok(cached.jwks.clone());
            }
        }
    }

    let mut keys = Vec::new();

    let mut rd = fs::read_dir(dir).await.map_err(|err| {
        error!("Failed to read public directory: {}", err);
        AuthTokenError::JwksFetchError
    })?;

    while let Some(entry) = rd.next_entry().await.map_err(|err| {
        error!("Failed to read directory entry: {}", err);
        AuthTokenError::JwksFetchError
    })? {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        if !fname.ends_with("_public.pem") {
            continue;
        }

        let kid = fname.trim_end_matches("_public.pem").to_string();
        let pem = fs::read_to_string(&path).await.map_err(|err| {
            error!("Failed to read public PEM file {:?}: {}", path, err);
            AuthTokenError::JwksFetchError
        })?;
        let b64 = pem
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<String>();
        let der = Base64::decode_vec(&b64).map_err(|err| {
            error!(
                "Failed to decode base64 content of PEM file {:?}: {}",
                path, err
            );
            AuthTokenError::JwksFetchError
        })?;
        let public_key = &der[der.len() - 32..];
        let x = Base64UrlUnpadded::encode_string(public_key);

        let jwk = Jwk {
            algorithm: AlgorithmParameters::OctetKeyPair(OctetKeyPairParameters {
                key_type: OctetKeyPairType::OctetKeyPair,
                curve: EllipticCurve::Ed25519,
                x,
            }),
            common: CommonParameters {
                key_id: Some(kid),
                ..Default::default()
            },
        };
        keys.push(jwk);
    }

    let jwks = JwkSet { keys };

    {
        let mut write = CACHE.write().await;
        *write = Some(CacheEntry {
            jwks: jwks.clone(),
            expires_at: Instant::now() + Duration::from_secs(CACHE_TTL_SECS),
        });
    }

    Ok(jwks)
}
