#![allow(dead_code)]

use std::env::var;
use std::fs::create_dir;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use walkdir::WalkDir;

use crate::constants::APP_NAME;
use crate::error::*;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Session {
    pub selected_profile: Option<i32>,
    pub selected_instance: Option<i32>,
    pub available_profiles: Option<Vec<String>>,
}

impl AsRef<Session> for Session {
    fn as_ref(&self) -> &Session {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Profile {
    pub name: String,
    pub game_path: PathBuf,
    pub instances: Option<Vec<Instance>>,
}

impl Profile {
    pub fn new(name: &str, game_path: PathBuf) -> Profile {
        Profile {
            name: name.to_owned(),
            game_path,
            instances: None,
        }
    }

    pub fn add_instance(&mut self, instance: Instance) {
        match &mut self.instances {
            Some(instances) => instances.push(instance),
            None => self.instances = Some(vec![instance]),
        }
    }

    pub fn remove_instance(&mut self, name: &str) {
        if let Some(instances) = &mut self.instances {
            instances.retain(|i| i.name != name);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Instance {
    pub name: String,
    pub files: Option<Vec<FileNode>>,
    pub mods: Option<Vec<ModInfo>>,
    pub downloads: Option<Vec<DownloadInfo>>,
}

impl Instance {
    pub fn new<P: AsRef<Path>>(name: &str, files_path: P) -> Instance {
        let mut files: Vec<FileNode> = Vec::new();

        for entry in WalkDir::new(files_path.as_ref()).into_iter().flatten() {
            let path = entry.path().to_path_buf();
            files.push(FileNode::new(
                path.clone(),
                path.file_name().unwrap().to_str().unwrap().to_string(),
                None,
                true,
            ))
        }

        Instance {
            name: name.to_owned(),
            mods: None,
            files: Some(files),
            downloads: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModInfo {
    pub name: String,
    pub path: PathBuf,
    pub config: ModConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModConfig {
    pub enabled: bool,
    pub files: Option<Vec<FileNode>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DownloadInfo {
    pub name: String,
    pub url: String,
    pub path: PathBuf,
    pub files: Option<Vec<FileNode>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Ord, PartialOrd)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub overriden_by: Option<(String, PathBuf)>,
    pub enabled: bool,
}

impl FileNode {
    pub fn new(path: PathBuf, name: String, overriden_by: Option<(String, PathBuf)>, enabled: bool) -> FileNode {
        FileNode {
            path,
            name,
            overriden_by,
            enabled,
        }
    }

    pub fn length(&self) -> usize {
        self.path.components().count()
    }

    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }

    pub fn override_by(&mut self, name: String, path: PathBuf) {
        self.overriden_by = Some((name, path));
    }

    pub fn into_tree_item(self, tree: &fltk::tree::Tree) -> fltk::tree::TreeItem {
        let mut item = fltk::tree::TreeItem::new(tree, &self.name);
        if self.enabled {
            item.set_user_icon(crate::constants::checked_icon());
            item.set_label_color(fltk::enums::Color::Green);
        } else {
            item.set_user_icon(crate::constants::unchecked_icon());
            if self.overriden_by.is_none() {
                item.set_label_color(fltk::enums::Color::Red);
            } else {
                item.set_user_data(self.overriden_by.unwrap());
                item.set_label_color(fltk::enums::Color::Yellow);
            }
        }
        item
    }
}

fn load_instances(path: PathBuf) -> Option<Vec<Instance>> {
    let mut instances: Option<Vec<Instance>> = None;
    let instance_directories = read_dir(path).ok()?;

    for instance_directory in instance_directories {
        let instance_directory_entry = instance_directory.ok()?;
        let mut sub_entries = read_dir(instance_directory_entry.path()).ok()?;

        let instance = sub_entries.find_map(|sub_entry| {
            sub_entry.ok().and_then(|sub_entry| {
                if !sub_entry.path().is_dir() && sub_entry.file_name().to_string_lossy().ends_with(".json") {
                    let instance_json = read_to_string(sub_entry.path()).ok()?;
                    let instance = serde_json::from_str(&instance_json).ok()?;
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

fn save_instances(instances: Vec<Instance>, path: PathBuf) -> Result<(), ProfileError> {
    for instance in instances {
        let instance_json =
            serde_json::to_string_pretty(&instance).map_err(|e| ProfileError::FailedToLoadInstances(format!("Kind: Json, Description: {}", e)))?;

        if !path.join(&instance.name).exists() {
            create_dir(path.join(&instance.name))?;
        }

        write(
            path.join(instance.name).join("instance.json"),
            instance_json,
        )
        .map_err(|e| ProfileError::FailedToLoadInstances(format!("Kind: {}, Description: {}", e.kind(), e)))?;
    }

    Ok(())
}

pub fn load_profile<P: AsRef<Path>>(name: &str, custom_path: Option<P>) -> Option<Profile> {
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

            if entry.path().is_dir() {
                return None;
            }

            if entry.file_name().to_string_lossy().to_lowercase() != "profile.json" {
                return None;
            }

            let profile_str = read_to_string(entry.path()).ok()?;

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

pub fn save_profile<P: AsRef<Path>>(profile: Profile, custom_path: Option<P>) -> Result<(), ProfileError> {
    let default_profile_path = match custom_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(local_app_data()).join(APP_NAME),
    };

    create_dir_all(default_profile_path.join(&profile.name).join("instances"))
        .map_err(|e| ProfileError::FailedToSaveProfile(format!("Kind: {}, Description: {}", e.kind(), e)))?;

    let profile_json =
        serde_json::to_string_pretty(&profile).map_err(|e| ProfileError::FailedToSaveProfile(format!("Kind: Json, Description: {}", e)))?;

    write(
        default_profile_path
            .join(&profile.name)
            .join("profile.json"),
        profile_json,
    )?;

    if let Some(instances) = &profile.instances {
        save_instances(
            instances.clone(),
            default_profile_path.join(&profile.name).join("instances"),
        )?;
    }

    Ok(())
}

pub fn init_profile<P: AsRef<Path>>(name: &str, game_path: P, init_path: Option<P>) -> Result<(), ProfileError> {
    let default_profile_path = match init_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(local_app_data()).join(APP_NAME),
    };

    create_dir_all(default_profile_path.join(name))
        .map_err(|e| ProfileError::FailedToInitProfile(format!("Kind: {}, Description: {}", e.kind(), e)))?;

    let profile = Profile {
        name: name.to_owned(),
        game_path: game_path.as_ref().to_path_buf(),
        instances: None,
    };

    let profile_json =
        serde_json::to_string_pretty(&profile).map_err(|e| ProfileError::FailedToInitProfile(format!("Kind: Json, Description: {}", e)))?;

    write(
        default_profile_path.join(name).join("profile.json"),
        profile_json,
    )?;

    Ok(())
}

pub fn save_session<P: AsRef<Path>>(
    selected_profile: Option<i32>,
    selected_instance: Option<i32>,
    available_profiles: Option<Vec<String>>,
    custom_path: Option<P>,
) -> Result<(), SessionError> {
    let session = Session {
        selected_profile,
        selected_instance,
        available_profiles,
    };

    let default_path = match custom_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(local_app_data()).join(APP_NAME),
    };

    let session_json = serde_json::to_string_pretty(&session)?;
    write(default_path.join("session.json"), session_json)?;

    Ok(())
}

pub fn load_session<P: AsRef<Path>>(custom_path: Option<P>) -> Option<Session> {
    let default_path = match custom_path {
        Some(p) => p.as_ref().to_path_buf(),
        None => PathBuf::from(local_app_data()).join(APP_NAME),
    };

    if !default_path.exists() {
        return None;
    }

    let session_json = read_to_string(default_path.join("session.json")).ok()?;

    let Ok(session): Result<Session, _> = serde_json::from_str(&session_json) else {
        return None;
    };

    Some(session)
}

#[macro_export]
macro_rules! save_session {
    ($selected_profile: expr, $selected_instance: expr, $available_profiles: expr, $custom_path: expr) => {
        $crate::profile::save_session(
            $selected_profile,
            $selected_instance,
            $available_profiles,
            $custom_path,
        )
    };
    ($selected_profile: expr, $selected_instance: expr, $available_profiles: expr) => {
        $crate::profile::save_session::<String>(
            $selected_profile,
            $selected_instance,
            $available_profiles,
            None,
        )
    };
}

#[macro_export]
macro_rules! load_session {
    () => {
        $crate::profile::load_session::<String>(None)
    };

    ($custom_path: expr) => {
        $crate::profile::load_session($custom_path)
    };
}

#[macro_export]
macro_rules! save_profile {
    ($profile: expr) => {
        $crate::profile::save_profile::<String>($profile, None)
    };
    ($profile: expr, $custom_path: expr) => {
        $crate::profile::save_profile($profile, $custom_path)
    };
}

#[macro_export]
macro_rules! load_profile {
    ($name: expr) => {
        $crate::profile::load_profile::<String>($name, None)
    };
    ($name: expr, $custom_path: expr) => {
        $crate::profile::load_profile($name, $custom_path)
    };
}

#[macro_export]
macro_rules! local_profiles {
    () => {
        std::path::PathBuf::from($crate::profile::local_app_data()).join($crate::constants::APP_NAME)
    };
}

#[macro_export]
macro_rules! local_instances {
    ($profile_name: expr) => {
        std::path::PathBuf::from($crate::profile::local_app_data())
            .join($crate::constants::APP_NAME)
            .join($profile_name)
            .join("instances")
    };
}
