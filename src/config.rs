use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ApplicationPreferences {
    pub mod_storage_dir: PathBuf,
    pub theme_name: String,
}
