use application::port::for_auth_tokens::AuthTokenError;
use log::error;
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{fs, sync::RwLock};

static CACHE: Lazy<RwLock<Option<Cached>>> = Lazy::new(|| RwLock::new(None));

struct Cached {
    file_name_no_pem: String,
    bytes: Arc<Vec<u8>>,
    loaded_at: Instant,
}

const TTL: Duration = Duration::from_secs(10 * 60);

pub async fn get_first_key_cached(dir: &str) -> Result<(String, Arc<Vec<u8>>), AuthTokenError> {
    {
        let read = CACHE.read().await;
        if let Some(cached) = read.as_ref() {
            if cached.loaded_at.elapsed() < TTL {
                return Ok((cached.file_name_no_pem.clone(), cached.bytes.clone()));
            }
        }
    }

    let mut entries: Vec<PathBuf> = {
        let mut rd = fs::read_dir(dir).await.map_err(|err| {
            error!("Failed to read keys directory: {}", err);
            AuthTokenError::TokenCreationFailure
        })?;
        let mut v = Vec::new();
        while let Some(entry) = rd.next_entry().await.map_err(|err| {
            error!("Failed to read directory entry: {}", err);
            AuthTokenError::TokenCreationFailure
        })? {
            let fname_os = entry.file_name();
            let fname = fname_os.to_string_lossy();
            if fname.ends_with("_key.pem") {
                v.push(entry.path());
            }
        }
        v
    };

    if entries.is_empty() {
        error!("No key files found in the keys directory.");
        return Err(AuthTokenError::TokenCreationFailure);
    }

    entries.sort_by(|a, b| {
        let an = a
            .file_name()
            .map(|s| s.to_os_string())
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let bn = b
            .file_name()
            .map(|s| s.to_os_string())
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        bn.cmp(&an) // decrescente
    });

    let first_path = entries.into_iter().next().expect("checked empty already");

    let data = fs::read(&first_path).await.map_err(|err| {
        error!("Failed to read key file {:?}: {}", first_path, err);
        AuthTokenError::TokenCreationFailure
    })?;
    let arc = Arc::new(data);

    let file_name_no_pem = first_path
        .file_name()
        .and_then(|os| os.to_str())
        .map(|s| {
            if s.ends_with("_key.pem") {
                let trimmed = &s[..s.len() - 8];
                trimmed.to_string()
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| String::new());

    {
        let mut write = CACHE.write().await;
        *write = Some(Cached {
            file_name_no_pem: file_name_no_pem.clone(),
            bytes: arc.clone(),
            loaded_at: Instant::now(),
        });
    }

    Ok((file_name_no_pem, arc))
}
