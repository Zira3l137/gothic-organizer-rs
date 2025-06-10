use fltk::prelude::FltkError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GothicOrganizerError {
    #[error("Profile Error: {0}")]
    Profile(#[from] ProfileError),
}

#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Failed to load instances: {0}")]
    FailedToLoadInstances(String),
    #[error("Failed to save profile: {0}")]
    FailedToSaveProfile(String),
    #[error("Failed to init profile: {0}")]
    FailedToInitProfile(String),
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum GuiError {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Fltk Error: {0}")]
    Fltk(#[from] FltkError),
    #[error("Profile Error: {0}")]
    Profile(#[from] ProfileError),
    #[error("Session Error: {0}")]
    Session(#[from] SessionError),
    #[error("Widget not found: {0}")]
    WidgetNotFound(String),
    #[error("Strip prefix error: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
}

impl GuiError {
    pub fn err_io<T>(msg: &str, kind: std::io::ErrorKind) -> Result<T, GuiError> {
        Err(GuiError::IO(std::io::Error::new(kind, msg)))
    }
}
