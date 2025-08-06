use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;
use serde::Serialize;
use crate::models::document::Document;
use crate::models::storage::Object;

#[async_trait]
pub trait StorageLayer<T>: Send + Sync  {
    async fn save(&mut self, document: Document<T>) -> Result<Object, Box<dyn std::error::Error>>;
    async fn get(&self, object: Object) -> Result<Object, Box<dyn std::error::Error>>;
    async fn list(&self, location: String) -> Result<Vec<Object>, Box<dyn std::error::Error>>;
    async fn delete(&mut self, object: Object) -> Result<(), Box<dyn std::error::Error>>;
}

// A storage layer meant for testing/development purposes only
pub(crate) struct LocalStorageLayer<T> {
    pub doc_store: HashMap<String, Document<T>>,
    pub object_store: HashMap<String, Object>,
    pub location: String,
}

impl<T: Serialize + Clone + Send + Sync> LocalStorageLayer<T> {
    pub fn new(location: &str) -> Self {
        LocalStorageLayer {
            doc_store: HashMap::new(),
            object_store: HashMap::new(),
            location: location.to_string(),
        }
    }
}

impl<T: Serialize + Clone + Send + Sync> StorageLayer<T> for LocalStorageLayer<T> {

    async fn save(&mut self, document: Document<T>) -> Result<Object, Box<dyn std::error::Error>> {
        let doc_id = document._id.clone();
        let doc_contents: Vec<u8> = document.contents.as_bytes().to_vec();
        let updated_at = Utc::now().to_string();
        if self.doc_store.contains_key(&doc_id) && self.object_store.contains_key(&doc_id) {
            let previous_object = self.object_store.get(&doc_id).unwrap();
            let object = Object::new(&previous_object.key, &self.location, &previous_object.created_at, &updated_at, doc_contents);
            self.object_store.insert(doc_id.as_str(), object.clone());
            self.doc_store.insert(doc_id.clone(), document);
            Ok(object)
        } else {
            let object_key = Uuid::default();
            let object = Object::new(&object_key.to_string(), &self.location, &updated_at, &updated_at, doc_contents);
            self.doc_store.insert(doc_id.clone(), document);
            self.object_store.insert(doc_id, object.clone());
            Ok(object)
        }
    }

    async fn get(&self, object: Object) -> Result<Object, Box<dyn std::error::Error>> {
        todo!()
    }

    async fn list(&self, location: String) -> Result<Vec<Object>, Box<dyn std::error::Error>> {
        todo!()
    }

    async fn delete(&mut self, object: Object) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

