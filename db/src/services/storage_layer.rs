use std::collections::HashMap;
use async_trait::async_trait;
use serde::Serialize;
use crate::models::document::Document;
use crate::models::storage::Object;

#[async_trait]
pub trait StorageLayer<T>: Send + Sync  {
    // if a document already existed in the same location with the same id, returns the old document
    // that has been overwritten
    async fn save(&mut self, document: Document<T>, location: &str) -> Result<Option<Document<T>>, Box<dyn std::error::Error>>;
    async fn get(&self, id: &str, location: &str) -> Result<Option<Document<T>>, Box<dyn std::error::Error>>;
    async fn list(&self, location: &str) -> Result<Vec<Document<T>>, Box<dyn std::error::Error>>;
    async fn delete(&mut self, id: &str, location: &str) -> Result<Document<T>, Box<dyn std::error::Error>>;
}

// A storage layer meant for testing/development purposes only simulating object storage (e.g. s3)
pub(crate) struct InMemoryStorageLayer<T> {
    pub doc_store: HashMap<String, HashMap<String, Document<T>>>,
}

impl<T> InMemoryStorageLayer<T>
where
    T: Serialize + Clone + Send + Sync,
{
    pub fn new() -> Self {
        InMemoryStorageLayer {
            doc_store: HashMap::new(),
        }
    }
}

#[async_trait]
impl<T> StorageLayer<T> for InMemoryStorageLayer<T>
where
    T: Serialize + Clone + Send + Sync,
{

    async fn save(&mut self, document: Document<T>, location: &str) -> Result<Option<Document<T>>, Box<dyn std::error::Error>> {
        let doc_id = document._id.clone();
        if !self.doc_store.contains_key(location) {
            self.doc_store.insert(location.to_string(), HashMap::new());
        }
        let mut store = self.doc_store.get(location).unwrap();

        if store.contains_key(&doc_id) {
            let previous_doc = store.get(&doc_id).unwrap().clone();
            store.insert(doc_id, document);
            Ok(Some(previous_doc))
        } else {
            store.insert(doc_id.clone(), document);
            Ok(None)
        }
    }

    async fn get(&self, id: &str, location: &str) -> Result<Option<Document<T>>, Box<dyn std::error::Error>> {
        if self.doc_store.contains_key(location) {
            let store = self.doc_store.get(location).unwrap();
            match store.get(id) {
                Some(doc) => Ok(Some(doc.clone())),
                None => Ok(None)
            }
        } else {
           Ok(None)
        }
    }

    async fn list(&self, location: &str) -> Result<Vec<Document<T>>, Box<dyn std::error::Error>> {
        if !self.doc_store.contains_key(location) {
            return Ok(Vec::new())
        }
        let store = self.doc_store.get(location).unwrap();
        Ok(store.values().cloned().collect())
    }

    async fn delete(&mut self, id: &str, location: &str) -> Result<Document<T>, Box<dyn std::error::Error>> {
        if !self.doc_store.contains_key(location) {
            return Err(format!("location {location} not found").into())
        }
        let mut store = self.doc_store.get(location).unwrap();
        if !store.contains_key(id) {
            return Err(format!("location {location} does not contain document with id {id}").into())
        }
        let doc = store.get(id).unwrap().clone();
        store.remove(id);
        Ok(doc)
    }

}
