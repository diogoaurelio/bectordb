use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use serde::Serialize;

mod services;
mod models;

use crate::models::document::Document;
use crate::models::error::DBError;
use crate::services::storage_layer::StorageLayer;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

struct BectorDB<T> {
    storage: Arc<Mutex<dyn StorageLayer<T>>>,
    index: Arc<RwLock<HashMap<String, Document<T>>>>,
}

impl<T: Serialize + Clone> BectorDB<T> {

    pub fn new(storage: Box<dyn StorageLayer<T>>) -> Self {
        BectorDB{
            storage: Arc::new(Mutex::new(storage)),
            index: Arc::new(Default::default()),
        }
    }

    pub fn read(&self, id: &str) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.read() {
            Ok(read_store) => Ok(read_store.get(id).cloned()),
            Err(err) => {
                let err_msg = format!("Unable to acquire read access to index for read op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    pub fn create(&self, document: Document<T>) -> Result<(), DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                if write_store.contains_key(&document._id) {
                    return Err(DBError::DocumentExists("".to_string()))
                }
                write_store.insert(document._id.clone(), document);
                Ok(())
            },
            Err(err) => {
                let err_msg = format!("Unable to acquire write access to index for create op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    // Updates an existing key, returns previous value that was there stored
    pub fn update(&self, document: Document<T>) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                if !write_store.contains_key(&document._id) {
                    let err_msg = format!("Document with id {} not found", &document._id);
                    return Err(DBError::NotFound(err_msg))
                }
                Ok(write_store.insert(document._id.clone(), document))
            },
            Err(err) => {
                let err_msg = format!("Unable to acquire write access to index for upsert op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    // inserts if key does not exist, updates if it did exist; if it did exist, returns old value
    pub fn upsert(&self, document: Document<T>) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                Ok(write_store.insert(document._id.clone(), document))
            },
            Err(err) => {
                let err_msg = format!("Unable to acquire write access to index for upsert op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    // deletes if a key exists; returns document that was previously associated w the key
    pub fn delete(&self, id: &str) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                if !write_store.contains_key(id) {
                    let err_msg = format!("Key {id} not found");
                    return Err(DBError::NotFound(err_msg))
                }
                Ok(write_store.remove(id))
            },
            Err(err) => {
                let err_msg = format!("Unable to acquire write access to index for upsert op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::models::storage::Object;
    use crate::services::storage_layer::LocalStorageLayer;
    use super::*;

    fn get_sut(location: &str) -> BectorDB<String> {
        let storage: LocalStorageLayer<String> = LocalStorageLayer::new(location);
        BectorDB::new(Box::new(storage))
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
