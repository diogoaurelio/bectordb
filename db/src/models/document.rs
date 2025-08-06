use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::TimestampMilliSeconds;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document<T> {
    pub _id: String,
    pub properties: HashMap<String, String>,
    pub embedding: Vec<f64>,
    pub contents: T,
    #[serde_as(as = "TimestampMilliSeconds")]
    pub created_at: DateTime<Utc>,
    #[serde_as(as = "TimestampMilliSeconds")]
    pub updated_at: DateTime<Utc>,
}
