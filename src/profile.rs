#![allow(dead_code)]

use std::collections::HashMap;
use std::env::var;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::constants::APP_NAME;
use crate::error::InitProfileError;

pub fn local_app_data() -> String {
    #[cfg(windows)]
    {
        var("LOCALAPPDATA").unwrap_or(String::from(""))
    }
    #[cfg(unix)]
    {
        var("XDG_DATA_HOME").unwrap_or(String::from("~/.local/share"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub game_path: PathBuf,
    pub instances: Option<Vec<Instance>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub mods: Option<Vec<ModInfo>>,
    pub downloads: Option<Vec<DownloadInfo>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub path: PathBuf,
    pub config: ModConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModConfig {
    pub enabled: bool,
    pub files: Option<HashMap<PathBuf, bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub name: String,
    pub url: String,
    pub path: PathBuf,
    pub files: Option<Vec<PathBuf>>,
}

pub fn load_instances(path: PathBuf) -> Option<Vec<Instance>> {
    let mut instances: Option<Vec<Instance>> = None;

    let instance_directories = read_dir(path).ok()?;

    for instance_directory in instance_directories {
        let Some(instance_directory_entry) = instance_directory.ok() else {
            continue;
        };

        let Some(mut sub_entries) = read_dir(instance_directory_entry.path()).ok() else {
            continue;
        };

        let instance = sub_entries.find_map(|sub_entry| {
            sub_entry.ok().and_then(|sub_entry| {
                if !sub_entry.path().is_dir() && sub_entry.file_name().to_string_lossy().ends_with(".json") {
                    let Ok(instance_json) = read_to_string(sub_entry.path()) else {
                        return None;
                    };

                    let Ok(instance) = serde_json::from_str(&instance_json) else {
                        return None;
                    };

                    Some(instance)
                } else {
                    None
                }
            })
        });

        if let Some(instance) = instance {
            if let Some(instances) = &mut instances {
                instances.push(instance);
            } else {
                instances = Some(vec![instance]);
            }
        }
    }

    instances
}

pub fn load_profile<P: AsRef<Path>>(name: String, custom_path: Option<P>) -> Option<Profile> {
    let default_profile_path = match custom_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(local_app_data()).join(APP_NAME),
    };

    let mut entries = read_dir(default_profile_path).ok()?;

    let profile = entries.find_map(|e| {
        let entry = e.ok()?;

        if !entry.path().is_dir() {
            return None;
        }

        if entry.file_name().to_string_lossy().to_lowercase() != name.to_lowercase() {
            return None;
        }

        let mut profile_dir = read_dir(entry.path()).ok()?;

        let profile_str = profile_dir.find_map(|e| {
            let entry = e.ok()?;

            if !entry.path().is_file() {
                return None;
            }

            if entry.file_name().to_string_lossy().to_lowercase() != "profile.json" {
                return None;
            }

            let Ok(profile_str) = read_to_string(entry.path()) else {
                return None;
            };

            Some(profile_str)
        })?;

        let Ok(mut profile): Result<Profile, _> = serde_json::from_str(&profile_str) else {
            return None;
        };

        profile.instances = load_instances(entry.path().join("instances"));

        Some(profile)
    })?;

    Some(profile)
}

pub fn init_profile<P: AsRef<Path>>(name: String, game_path: P, init_path: Option<P>) -> Result<(), InitProfileError> {
    let default_profile_path = match init_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(local_app_data()).join(APP_NAME),
    };

    create_dir_all(default_profile_path.join(&name))?;

    let profile = Profile {
        name: name.clone(),
        game_path: game_path.as_ref().to_path_buf(),
        instances: None,
    };

    let profile_json = serde_json::to_string_pretty(&profile)?;

    write(
        default_profile_path.join(&name).join("profile.json"),
        profile_json,
    )?;

    Ok(())
}
