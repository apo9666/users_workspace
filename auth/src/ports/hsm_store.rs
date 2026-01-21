use async_trait::async_trait;
use contracts::auth::error::HSMStoreError;
use uuid::Uuid;

#[async_trait]
pub trait HSMStore: Send + Sync {
    fn get(&self, user_id: Uuid, key: &str) -> Result<Option<String>, HSMStoreError>;
    fn set(&self, user_id: Uuid, key: &str, value: &str) -> Result<(), HSMStoreError>;
}
