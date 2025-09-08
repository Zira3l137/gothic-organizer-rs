#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;

pub type Lookup<K, V> = hashbrown::HashMap<K, V, ahash::RandomState>;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub instances: Option<Lookup<String, Instance>>,
}

impl Profile {
    pub fn new(name: &str, path: &Path) -> Self {
        Self { name: name.to_owned(), path: path.to_owned(), instances: None }
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
            match instances.entry(instance.name.clone()) {
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
            self.instances = Some(std::iter::once((instance.name.clone(), instance)).collect());
        }
    }

    pub fn remove_instance(&mut self, instance_name: &str) {
        if let Some(instances) = self.instances.as_mut() {
            instances.remove(instance_name);
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance {
    pub name: String,
    pub files: Lookup<PathBuf, FileMetadata>,
    pub conflicts: Conflicts,
    pub mods: Vec<ModInfo>,
    pub load_order: Lookup<String, usize>,
}

impl Instance {
    pub fn new(name: &str, files: Lookup<PathBuf, FileMetadata>, mods: Vec<ModInfo>) -> Self {
        Self {
            name: name.to_owned(),
            files,
            conflicts: Conflicts::default(),
            mods,
            load_order: Lookup::default(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn with_files(mut self, files: Lookup<PathBuf, FileMetadata>) -> Self {
        self.files = files;
        self
    }

    pub fn with_mods(mut self, mods: Vec<ModInfo>) -> Self {
        self.mods = mods;
        self
    }

    pub fn with_conflicts(mut self, conflicts: Conflicts) -> Self {
        self.conflicts = conflicts;
        self
    }

    pub fn with_load_order(mut self, load_order: Lookup<String, usize>) -> Self {
        self.load_order = load_order;
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Conflicts {
    pub entries: Lookup<PathBuf, Lookup<usize, FileMetadata>>,
}

impl Conflicts {
    pub fn new<T>(entries: T) -> Self
    where
        T: Into<Lookup<PathBuf, Lookup<usize, FileMetadata>>>,
    {
        Self { entries: entries.into() }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &Lookup<usize, FileMetadata>)> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&PathBuf, &mut Lookup<usize, FileMetadata>)> {
        self.entries.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ModInfo {
    pub enabled: bool,
    pub name: String,
    pub path: PathBuf,
    pub files: Lookup<PathBuf, FileMetadata>,
}

impl ModInfo {
    pub fn new(enabled: bool, name: &str, path: &Path, files: Lookup<PathBuf, FileMetadata>) -> Self {
        Self { enabled, name: name.to_owned(), path: path.to_owned(), files }
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

    pub fn with_files(mut self, files: Lookup<PathBuf, FileMetadata>) -> Self {
        self.files = files;
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    pub enabled: bool,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub parent_name: String,
}

impl FileMetadata {
    pub fn new(enabled: bool, source_path: &Path, target_path: &Path, parent_name: &str) -> Self {
        Self {
            enabled,
            source_path: source_path.to_owned(),
            target_path: target_path.to_owned(),
            parent_name: parent_name.to_owned(),
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

    pub fn with_parent_name(mut self, parent_name: &str) -> Self {
        self.parent_name = parent_name.to_owned();
        self
    }
}
