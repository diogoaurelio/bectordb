use std::collections::HashMap;
use async_trait::async_trait;
use serde::Serialize;
use crate::models::document::Document;

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
        let mut store = self.doc_store.get_mut(location).unwrap();

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
        let mut store = self.doc_store.get_mut(location).unwrap();
        if !store.contains_key(id) {
            return Err(format!("location {location} does not contain document with id {id}").into())
        }
        let doc = store.get(id).unwrap().clone();
        store.remove(id);
        Ok(doc)
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::models::document::Document;
    use crate::services::storage_layer::{InMemoryStorageLayer, StorageLayer};

    #[tokio::test]
    async fn get_method_should_return_a_document() {
        // given: a document w string type contents
        let id = "some-id";
        let contents = "some content".to_string();
        let properties: HashMap<String, String> = HashMap::new();
        let doc: Document<String> = Document::new(id, contents, properties);

        // given: a System Under Test (SUT) initialized with a map with an item already loaded
        let mut store = HashMap::new();
        let location = "some-location";
        store.insert(id.to_string(), doc.clone());
        let mut doc_store = HashMap::new();
        doc_store.insert(location.to_string(), store);

        let sut = InMemoryStorageLayer { doc_store };

        // when: method get from SUT is called
        let res = sut.get(id, location).await;

        // then: results should be as expected
        assert!(res.is_ok(), "it should successfully return ok result");
        let res = res.unwrap();
        assert!(res.is_some(), "a document should have been found");
        let res = res.unwrap();
        assert_eq!(doc._id, res._id, "it should have the same document id");
        assert_eq!(doc.contents, res.contents, "it should have the same document contents");

    }

    #[tokio::test]
    async fn save_method_should_not_return_a_document() {
        // given: a document w string type contents
        let id = "some-id";
        let contents = "some content".to_string();
        let properties: HashMap<String, String> = HashMap::new();
        let doc: Document<String> = Document::new(id, contents, properties);
        let location = "some-location";

        // given: a System Under Test (SUT) initialized with empty a map
        let mut sut = InMemoryStorageLayer::new();

        // when: method save from SUT is called
        let res = sut.save(doc, location).await;

        // then: results should be as expected
        assert!(res.is_ok(), "it should successfully return ok result");
        let res = res.unwrap();
        assert!(res.is_none(), "no document should have been found there previously");

    }

    #[tokio::test]
    async fn save_method_should_return_previously_existing_document() {
        // given: a document w string type contents
        let id = "some-id";
        let contents = "some content".to_string();
        let properties: HashMap<String, String> = HashMap::new();
        let previous_doc: Document<String> = Document::new(id, contents, properties.clone());

        // given: a System Under Test (SUT) initialized with a map with an item already loaded
        let mut store = HashMap::new();
        let location = "some-location";
        store.insert(id.to_string(), previous_doc.clone());
        let mut doc_store = HashMap::new();
        doc_store.insert(location.to_string(), store);

        let mut sut = InMemoryStorageLayer { doc_store };


        // when: method save from SUT is called with a new document
        let new_contents = "new-contents".to_string();
        let new_doc: Document<String> = Document::new(id, new_contents, properties);
        let res = sut.save(previous_doc.clone(), location).await;

        // then: results should be as expected
        assert!(res.is_ok(), "it should successfully return ok result");
        let res = res.unwrap();
        assert!(res.is_some(), "the document previously saved with the same id should be returned");
        let res = res.unwrap();
        assert_eq!(previous_doc._id, res._id, "it should have the same document id");
        assert_eq!(previous_doc.contents, res.contents, "it should have the same document contents as the previously present document");

    }

    #[tokio::test]
    async fn delete_method_should_return_previously_existing_document() {
        // given: a document w string type contents
        let id = "some-id";
        let contents = "some content".to_string();
        let properties: HashMap<String, String> = HashMap::new();
        let previous_doc: Document<String> = Document::new(id, contents, properties.clone());

        // given: a System Under Test (SUT) initialized with a map with an item already loaded
        let mut store = HashMap::new();
        let location = "some-location";
        store.insert(id.to_string(), previous_doc.clone());
        let mut doc_store = HashMap::new();
        doc_store.insert(location.to_string(), store);

        let mut sut = InMemoryStorageLayer { doc_store };

        // when: method delete from SUT is called with a new document
        let new_contents = "new-contents".to_string();
        let new_doc: Document<String> = Document::new(id, new_contents, properties);
        let res = sut.delete(id, location).await;

        // then: results should be as expected
        assert!(res.is_ok(), "it should successfully return ok result");
        let res = res.unwrap();
        assert_eq!(previous_doc._id, res._id, "it should have the same document id");
        assert_eq!(previous_doc.contents, res.contents, "it should have the same document contents as the previously present document");

    }

}
