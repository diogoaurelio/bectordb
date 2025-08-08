use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document<T> {
    pub _id: String,
    pub properties: HashMap<String, String>,
    pub embedding: Vec<f64>,
    pub contents: T,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl<T: Serialize + Clone> Document<T> {
    pub fn new(id: &str, contents: T, properties: HashMap<String, String>) -> Self {
        Document {
            _id: id.to_string(),
            contents,
            properties,
            embedding: vec![],
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}
