use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Object {
    pub contents: Vec<u8>,
    pub created_at: String,
    pub key: String,
    pub location: String,
    pub updated_at: String,
}

impl Object {
    pub fn new(key: &str, location: &str, created_at: &str, updated_at: &str, contents: Vec<u8>) -> Self {
        Object {
            contents,
            created_at: created_at.to_string(),
            key: key.to_string(),
            location: location.to_string(),
            updated_at: updated_at.to_string(),
        }
    }
}
