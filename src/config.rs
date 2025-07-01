use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::core::lookup::Lookup;
use crate::core::profile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub mod_storage_dir: PathBuf,
    pub theme: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Session {
    pub selected_profile: Option<String>,
    pub selected_instance: Option<String>,
    pub cache: Option<Lookup<PathBuf, profile::FileInfo>>,
}

impl AsRef<Session> for Session {
    fn as_ref(&self) -> &Session {
        self
    }
}
