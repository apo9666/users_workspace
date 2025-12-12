use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub enum HSMStoreError {
    StorageError(String),
}

impl std::fmt::Display for HSMStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HSMStoreError::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for HSMStoreError {}

#[async_trait]
pub trait HSMStore: Send + Sync {
    fn get(&self, user_id: Uuid, key: &str) -> Result<Option<String>, HSMStoreError>;
    fn set(&self, user_id: Uuid, key: &str, value: &str) -> Result<(), HSMStoreError>;
}
