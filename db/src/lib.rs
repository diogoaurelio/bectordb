use log::error;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

mod models;
mod services;

use crate::models::document::Document;
use crate::models::error::DBError;
use crate::services::storage_layer::StorageLayer;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

struct BectorDB<T> {
    storage: Arc<Mutex<Box<dyn StorageLayer<T>>>>,
    index: Arc<RwLock<HashMap<String, String>>>,
}

impl<T> BectorDB<T>
where
    T: Serialize + Clone + Send + Sync,
{
    pub fn new(storage: Box<dyn StorageLayer<T>>) -> Self {
        BectorDB {
            storage: Arc::new(Mutex::new(storage)),
            index: Arc::new(Default::default()),
        }
    }

    pub async fn read(&self, id: &str) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.read() {
            Ok(read_store) => {
                let location = read_store.get(id);
                if location.is_none() {
                    return Ok(None);
                }
                let location = location.unwrap();
                let storage = self.storage.lock().map_err(|err| {
                    let err_msg =
                        format!("Unable to acquire read access to storage for read op: {err}");
                    DBError::UnableToAcquireIndexLock(err_msg)
                })?;

                Ok(storage.get(id, location).await.map_err(|err| {
                    error!("error retrieving doc id {id}: {err}");
                    DBError::InternalServerError(
                        "unable to retrieve document".to_string(),
                    )
                })?)
            }
            Err(err) => {
                let err_msg = format!("Unable to acquire read access to index for read op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    pub async fn create(&self, document: Document<T>, collection: &str) -> Result<(), DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                if write_store.contains_key(&document._id) {
                    return Err(DBError::DocumentExists("".to_string()));
                }
                write_store.insert(document._id.clone(), collection.to_string());
                let mut storage = self.storage
                    .lock()
                    .map_err(|err| {
                        let err_msg =
                            format!("Unable to acquire read access to storage for read op: {err}");
                        DBError::UnableToAcquireIndexLock(err_msg)
                    })?;
                Ok(storage
                    .save(document, collection)
                    .await
                    .map(|doc| { () })
                    .map_err(|err| {
                        let err_msg =
                            format!("unable to save document in collection {collection}: {err}");
                        DBError::InternalServerError(err_msg)
                    })?)
            }
            Err(err) => {
                let err_msg =
                    format!("Unable to acquire write access to index for create op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    // Updates an existing key, returns previous value that was there stored
    pub async fn update(&self, document: Document<T>, collection: &str) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                if !write_store.contains_key(&document._id) {
                    let err_msg = format!("Document with id {} not found", &document._id);
                    return Err(DBError::NotFound(err_msg));
                }
                let _ = write_store.insert(document._id.clone(), collection.to_string());

                let mut storage = self.storage
                    .lock()
                    .map_err(|err| {
                        let err_msg =
                            format!("Unable to acquire read access to storage for read op: {err}");
                        DBError::UnableToAcquireIndexLock(err_msg)
                    })?;
                Ok(storage
                    .save(document, collection)
                    .await
                    .map_err(|err| {
                        let err_msg =
                            format!("unable to save document in collection {collection}: {err}");
                        DBError::InternalServerError(err_msg)
                    })?)
            }
            Err(err) => {
                let err_msg =
                    format!("Unable to acquire write access to index for upsert op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    // inserts if key does not exist, updates if it did exist; if it did exist, returns old value
    pub async fn upsert(&self, document: Document<T>, collection: &str) -> Result<Option<Document<T>>, DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                write_store.insert(document._id.clone(), collection.to_string());
                let mut storage = self.storage
                    .lock()
                    .map_err(|err| {
                        let err_msg =
                            format!("Unable to acquire read access to storage for read op: {err}");
                        DBError::UnableToAcquireIndexLock(err_msg)
                    })?;
                Ok(storage
                    .save(document, collection)
                    .await
                    .map_err(|err| {
                        let err_msg =
                            format!("unable to save document in collection {collection}: {err}");
                        DBError::InternalServerError(err_msg)
                    })?)
            },
            Err(err) => {
                let err_msg =
                    format!("Unable to acquire write access to index for upsert op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }

    // deletes if a key exists; returns document that was previously associated w the key
    pub async fn delete(&self, id: &str, collection: &str) -> Result<Document<T>, DBError> {
        let index = Arc::clone(&self.index);
        match index.write() {
            Ok(mut write_store) => {
                if !write_store.contains_key(id) {
                    let err_msg = format!("Key {id} not found");
                    return Err(DBError::NotFound(err_msg));
                }

                let mut storage = self.storage
                    .lock()
                    .map_err(|err| {
                        let err_msg =
                            format!("Unable to acquire read access to storage for read op: {err}");
                        DBError::UnableToAcquireIndexLock(err_msg)
                    })?;
                let doc = storage
                    .delete(id, collection)
                    .await
                    .map_err(|err| {
                        let err_msg =
                            format!("unable to remove document in collection {collection}: {err}");
                        DBError::InternalServerError(err_msg)
                    })?;
                write_store.remove(id);
                Ok(doc)
            }
            Err(err) => {
                let err_msg =
                    format!("Unable to acquire write access to index for upsert op: {err}");
                Err(DBError::UnableToAcquireIndexLock(err_msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::storage_layer::InMemoryStorageLayer;

    fn get_sut(storage: Box<dyn StorageLayer<String>>) -> BectorDB<String> {
        BectorDB::new(storage)
    }

    #[tokio::test]
    async fn get_method_should_return_a_document() {
        // given: a document w string type contents
        let id = "some-id";
        let contents = "some content".to_string();
        let properties: HashMap<String, String> = HashMap::new();
        let doc: Document<String> = Document::new(id, contents, properties);
        let doc_location = "some-location";

        // given: a System Under Test (SUT) initialized with a map with an item already loaded
        let mut storage: InMemoryStorageLayer<String> = InMemoryStorageLayer::new();
        let _ = storage.save(doc.clone(), doc_location).await;
        let mut index = HashMap::new();
        index.insert(id.to_string(), doc_location.to_string());
        let index = Arc::new(RwLock::new(index));

        // given: a System Under Test (SUT) initialized with storage layer loaded with a doc
        let sut = BectorDB {
            storage: Arc::new(Mutex::new(Box::new(storage))),
            index,
        };

        // when: method get from SUT is called
        let res = sut.read(id).await;

        // then: results should be as expected
        assert!(res.is_ok(), "it should successfully return ok result");
        let res = res.unwrap();
        assert!(res.is_some(), "a document should have been found");
        let res = res.unwrap();
        assert_eq!(doc._id, res._id, "it should have the same document id");
        assert_eq!(
            doc.contents, res.contents,
            "it should have the same document contents"
        );
    }
}
