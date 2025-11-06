use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum RepositoryError {
    ConnectionError(String),
    Other(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            RepositoryError::Other(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl Error for RepositoryError {}
