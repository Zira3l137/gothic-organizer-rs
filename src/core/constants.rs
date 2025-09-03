#![allow(dead_code)]

use derive_more::Display;
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
    format!("{APP_TITLE}\nVersion: {APP_VERSION}\nAuthors: {APP_AUTHORS}\nRepository: {APP_REPOSITORY}")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum DefaultProfile {
    #[display("Gothic")]
    Gothic,
    #[display("Gothic 2 Classic")]
    Gothic2Classic,
    #[display("Gothic 2 Night of Raven")]
    Gothic2NightOfRaven,
    #[display("Gothic 2 Sequel")]
    GothicSequel,
}

impl DefaultProfile {
    pub fn into_iter() -> std::slice::Iter<'static, DefaultProfile> {
        static PROFILES: [DefaultProfile; 4] = [
            DefaultProfile::Gothic,
            DefaultProfile::Gothic2Classic,
            DefaultProfile::Gothic2NightOfRaven,
            DefaultProfile::GothicSequel,
        ];
        PROFILES.iter()
    }
}
