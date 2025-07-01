#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::core::lookup::Lookup;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub instances: Option<Lookup<String, Instance>>,
}

impl Profile {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_owned(),
            path: path.to_owned(),
            instances: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_path(mut self, path: &Path) -> Self {
        self.path = path.to_owned();
        self
    }

    pub fn with_instances(mut self, instances: Option<Lookup<String, Instance>>) -> Self {
        self.instances = instances;
        self
    }

    pub fn add_instance(&mut self, instance: Instance) {
        if let Some(instances) = self.instances.as_mut() {
            match instances.access.entry(instance.name.clone()) {
                hashbrown::hash_map::Entry::Occupied(mut entry) => {
                    let mut_value = entry.get_mut();
                    mut_value.name = instance.name;
                    mut_value.mods = instance.mods;
                }
                hashbrown::hash_map::Entry::Vacant(entry) => {
                    entry.insert(instance);
                }
            }
        } else {
            self.instances = Some(Lookup::from(vec![(instance.name.clone(), instance)]));
        }
    }

    pub fn remove_instance(&mut self, instance_name: &str) {
        if let Some(instances) = self.instances.as_mut() {
            instances.access.remove(instance_name);
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub files: Option<Lookup<PathBuf, FileInfo>>,
    pub overwrtites: Option<Lookup<PathBuf, FileInfo>>,
    pub mods: Option<Vec<ModInfo>>,
}

impl Instance {
    pub fn new(name: &str, files: Option<Lookup<PathBuf, FileInfo>>, mods: Option<Vec<ModInfo>>) -> Self {
        Self {
            name: name.to_owned(),
            files,
            overwrtites: None,
            mods,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_files(mut self, files: Option<Lookup<PathBuf, FileInfo>>) -> Self {
        self.files = files;
        self
    }

    pub fn with_mods(mut self, mods: Option<Vec<ModInfo>>) -> Self {
        self.mods = mods;
        self
    }

    pub fn with_overwrtites(mut self, overwrtites: Option<Lookup<PathBuf, FileInfo>>) -> Self {
        self.overwrtites = overwrtites;
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModInfo {
    pub enabled: bool,
    pub name: String,
    pub path: PathBuf,
    pub files: Lookup<PathBuf, FileInfo>,
}

impl ModInfo {
    pub fn new(enabled: bool, name: &str, path: &Path, files: Lookup<PathBuf, FileInfo>) -> Self {
        Self {
            enabled,
            name: name.to_owned(),
            path: path.to_owned(),
            files,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_path(mut self, path: &Path) -> Self {
        self.path = path.to_owned();
        self
    }

    pub fn with_files(mut self, files: Lookup<PathBuf, FileInfo>) -> Self {
        self.files = files;
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInfo {
    pub enabled: bool,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub parent_name: Option<String>,
}

impl FileInfo {
    pub fn new(enabled: bool, source_path: &Path, target_path: &Path, parent_name: Option<String>) -> Self {
        Self {
            enabled,
            source_path: source_path.to_owned(),
            target_path: target_path.to_owned(),
            parent_name,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_source_path(mut self, source_path: &Path) -> Self {
        self.source_path = source_path.to_owned();
        self
    }

    pub fn with_target_path(mut self, target_path: &Path) -> Self {
        self.target_path = target_path.to_owned();
        self
    }

    pub fn with_parent_name(mut self, parent_name: String) -> Self {
        self.parent_name = Some(parent_name);
        self
    }
}
