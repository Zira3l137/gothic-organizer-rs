#![allow(dead_code)]

use thiserror::Error;

use crate::impl_shared_error_from;

#[derive(Error, Debug)]
pub enum GothicOrganizerError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Error: {0}")]
    Other(String),
    #[error("Zip Error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

impl GothicOrganizerError {
    pub fn new(message: &str) -> Self {
        Self::Other(message.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SharedError(std::sync::Arc<dyn std::error::Error + Send + Sync>);

impl SharedError {
    pub fn new<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(std::sync::Arc::new(error))
    }
}

impl std::fmt::Display for SharedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for SharedError {}

impl_shared_error_from!(
    GothicOrganizerError,
    std::io::Error,
    serde_json::Error,
    zip::result::ZipError,
);
