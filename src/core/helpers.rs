use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;

use crate::app::session;
use crate::core::constants;
use crate::core::profile;

fn default_path<P: AsRef<Path>>(custom_path: Option<P>) -> PathBuf {
    match custom_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => crate::core::constants::local_app_data_path().join(constants::APP_NAME),
    }
}

pub fn save_app_session<P: AsRef<Path>>(
    session: &session::ApplicationSession,
    custom_path: Option<P>,
) -> Result<(), std::io::Error> {
    let default_path = default_path(custom_path);
    let session_string = serde_json::to_string_pretty(session)?;
    write(default_path.join("session.json"), session_string)?;

    Ok(())
}

pub fn load_app_session<P: AsRef<Path>>(custom_path: Option<P>) -> Option<session::ApplicationSession> {
    let default_path = default_path(custom_path);
    if !default_path.exists() {
        return None;
    }

    let session_json = read_to_string(default_path.join("session.json")).ok()?;

    let Ok(session): Result<session::ApplicationSession, _> = serde_json::from_str(&session_json) else {
        return None;
    };

    Some(session)
}

pub fn save_profile<P: AsRef<Path>>(
    profile: &profile::Profile,
    custom_path: Option<P>,
) -> Result<(), std::io::Error> {
    let default_profile_path = default_path(custom_path);
    let profile_json = serde_json::to_string_pretty(&profile).map_err(std::io::Error::other)?;

    create_dir_all(default_profile_path.join(&profile.name)).map_err(|e| std::io::Error::new(e.kind(), e))?;
    write(default_profile_path.join(&profile.name).join("profile.json"), profile_json)?;

    Ok(())
}

pub fn load_profile<P: AsRef<Path>>(name: &str, custom_path: Option<P>) -> Option<profile::Profile> {
    let default_profile_path = default_path(custom_path);
    let mut entries = read_dir(default_profile_path).ok()?;

    let profile = entries.find_map(|e| {
        let entry = e.ok()?;
        if !entry.path().is_dir() || entry.file_name().to_string_lossy().to_lowercase() != name.to_lowercase()
        {
            return None;
        }

        let mut profile_dir = read_dir(entry.path()).ok()?;
        let profile_str = profile_dir.find_map(|e| {
            let entry = e.ok()?;
            if entry.path().is_dir() || entry.file_name().to_string_lossy().to_lowercase() != "profile.json" {
                return None;
            }

            let profile_str = read_to_string(entry.path()).ok()?;
            Some(profile_str)
        })?;

        let Ok(profile): Result<profile::Profile, _> = serde_json::from_str(&profile_str) else {
            return None;
        };

        Some(profile)
    })?;

    Some(profile)
}
