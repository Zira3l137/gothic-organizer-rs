use thiserror::Error;

#[derive(Error, Debug)]
pub enum GothicOrganizerError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Error: {0}")]
    Other(String),
}

impl GothicOrganizerError {
    pub fn new(message: &str) -> Self {
        Self::Other(message.to_string())
    }
}
