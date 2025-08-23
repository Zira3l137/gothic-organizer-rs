#![allow(dead_code)]

use std::path::PathBuf;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const APP_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const APP_TITLE: &str = "Gothic Organizer";

pub fn app_title_full() -> String {
    format!("{APP_TITLE} v{APP_VERSION}")
}

pub fn app_info() -> String {
    format!(
        "{APP_TITLE}\nVersion: {APP_VERSION}\nAuthors: {APP_AUTHORS}\nRepository: {APP_REPOSITORY}"
    )
}

pub fn local_app_data_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or(String::from("")))
    }

    #[cfg(target_os = "linux")]
    {
        PathBuf::from(std::env::var("XDG_DATA_HOME").unwrap_or(String::from("~/.local/share")))
    }
}

pub fn local_profiles_path() -> PathBuf {
    local_app_data_path().join(APP_NAME)
}

pub fn default_mod_storage_path() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    exe_dir.join("mods")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Gothic,
    Gothic2Classic,
    Gothic2NightOfTheRaven,
    GothicSequel,
}

impl Profile {
    pub fn into_iter() -> std::slice::Iter<'static, Profile> {
        static PROFILES: [Profile; 4] = [
            Profile::Gothic,
            Profile::Gothic2Classic,
            Profile::Gothic2NightOfTheRaven,
            Profile::GothicSequel,
        ];
        PROFILES.iter()
    }
}

impl From<Profile> for &'static str {
    fn from(value: Profile) -> Self {
        match value {
            Profile::Gothic => "Gothic",
            Profile::Gothic2Classic => "Gothic 2 Classic",
            Profile::Gothic2NightOfTheRaven => "Gothic 2 Night of the Raven",
            Profile::GothicSequel => "Gothic Sequel",
        }
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Gothic => write!(f, "Gothic"),
            Profile::Gothic2Classic => write!(f, "Gothic 2 Classic"),
            Profile::Gothic2NightOfTheRaven => write!(f, "Gothic 2 Night of the Raven"),
            Profile::GothicSequel => write!(f, "Gothic Sequel"),
        }
    }
}
