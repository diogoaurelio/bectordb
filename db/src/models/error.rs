use std::fmt;

#[derive(Debug)]
pub enum DBError {
    DocumentExists(String),
    InternalServerError(String),
    NotFound(String),
    Unauthorized(String),
    UnableToAcquireIndexLock(String),
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DBError::DocumentExists(msg) => write!(f, "Document already exists: {}", msg),
            DBError::InternalServerError(msg) => write!(f, "Internal Server Error: {}", msg),
            DBError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            DBError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            DBError::UnableToAcquireIndexLock(msg) => write!(f, "Unable to acquire index lock {}", msg),
        }
    }
}

impl std::error::Error for DBError {}
