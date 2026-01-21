use crate::ports::hsm_store::HSMStore;
use contracts::auth::error::HSMStoreError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct MemoryHsmStore {
    store: Arc<RwLock<HashMap<(Uuid, String), String>>>,
}

impl MemoryHsmStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl HSMStore for MemoryHsmStore {
    fn get(&self, user_id: Uuid, key: &str) -> Result<Option<String>, HSMStoreError> {
        let map = self
            .store
            .read()
            .map_err(|e| HSMStoreError::StorageError(format!("Mutex poisoned: {}", e)))?;

        Ok(map.get(&(user_id, key.to_string())).cloned())
    }

    fn set(&self, user_id: Uuid, key: &str, value: &str) -> Result<(), HSMStoreError> {
        let mut map = self
            .store
            .write()
            .map_err(|e| HSMStoreError::StorageError(format!("Mutex poisoned: {}", e)))?;

        map.insert((user_id, key.to_string()), value.to_string());

        Ok(())
    }
}
