use thiserror::Error;

#[derive(Error, Debug)]
pub enum GothicOrganizerError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
}
