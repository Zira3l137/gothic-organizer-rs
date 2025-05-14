use thiserror::Error;

#[derive(Error, Debug)]
pub enum GothicOrganizerError {
    #[error("Failed to init profile")]
    FailedToInitProfile(#[from] InitProfileError),
}

#[derive(Error, Debug)]
pub enum InitProfileError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
}
