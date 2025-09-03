#![allow(dead_code)]

use std::path;

use crate::core::profile;
use crate::core::profile::Lookup;

type Overwrites = Lookup<String, Lookup<profile::FileMetadata, profile::FileMetadata>>;
type Mods = Vec<profile::ModInfo>;
type Instances = Lookup<String, profile::Instance>;
type InstanceFiles = Lookup<path::PathBuf, profile::FileMetadata>;

#[derive(Debug)]
pub struct Context<'ctx> {
    pub active_profile: &'ctx mut profile::Profile,
    pub active_instance_name: String,
}

impl<'ctx> Context<'ctx> {
    pub fn new(profile: &'ctx mut profile::Profile, instance_name: &str) -> Self {
        Self { active_profile: profile, active_instance_name: instance_name.to_owned() }
    }

    pub fn instance(&self, instance_name: Option<&str>) -> Option<&profile::Instance> {
        self.active_profile
            .instances
            .as_ref()?
            .get(instance_name.unwrap_or_else(|| &self.active_instance_name))
    }

    pub fn instance_mut(&mut self, instance_name: Option<&str>) -> Option<&mut profile::Instance> {
        self.active_profile
            .instances
            .as_mut()?
            .get_mut(instance_name.unwrap_or_else(|| &self.active_instance_name))
    }

    pub fn instance_mods(&self) -> Option<&Mods> {
        self.instance(Some(&self.active_instance_name))?.mods.as_ref()
    }

    pub fn instance_mods_mut(&mut self) -> Option<&mut Mods> {
        self.instance_mut(Some(&self.active_instance_name.clone()))?.mods.as_mut()
    }

    pub fn instance_files(&self) -> Option<&InstanceFiles> {
        self.instance(Some(&self.active_instance_name))?.files.as_ref()
    }

    pub fn instance_files_mut(&mut self) -> Option<&mut InstanceFiles> {
        self.instance_mut(Some(&self.active_instance_name.clone()))?.files.as_mut()
    }

    pub fn instances(&self) -> Option<&Instances> {
        self.active_profile.instances.as_ref()
    }

    pub fn instances_mut(&mut self) -> Option<&mut Instances> {
        self.active_profile.instances.as_mut()
    }

    pub fn instances_empty(&self) -> bool {
        self.instances().map(|i| i.is_empty()).unwrap_or(true)
    }

    pub fn instance_names(&self) -> Vec<String> {
        self.active_profile
            .instances
            .as_ref()
            .map(|instances| instances.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear_instance_overwrites(&mut self) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.overridden_files = None;
        }
    }

    pub fn clear_instance_mods(&mut self) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.mods = None;
        }
    }

    pub fn clear_instance_files(&mut self) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.files = None;
        }
    }

    pub fn set_instance_mods(&mut self, mods: Mods) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.mods = Some(mods);
        }
    }

    pub fn set_instance_files(&mut self, files: Option<InstanceFiles>) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.files = files;
        }
    }

    pub fn contains_instance(&self, instance_name: &str) -> bool {
        self.active_profile
            .instances
            .as_ref()
            .map(|instances| instances.contains_key(instance_name))
            .unwrap_or(false)
    }

    /// Inserts new instance into profile. If the instance already exists, it will be overwritten
    /// and the old instance will be returned otherwise None will be returned.
    pub fn insert_instance(&mut self, instance: profile::Instance) -> Option<profile::Instance> {
        if let Some(instances) = self.instances_mut() {
            instances.insert(instance.name.clone(), instance)
        } else {
            self.active_profile.instances = Some(Instances::default());
            self.active_profile.instances.as_mut().map(|i| i.insert(instance.name.clone(), instance));
            None
        }
    }

    /// Removes an instance from the profile, returning it. Returns None if the instance does not
    /// exist.
    pub fn remove_instance(&mut self, instance_name: &str) -> Option<profile::Instance> {
        if let Some(instances) = self.instances_mut() {
            return instances.remove(instance_name);
        }
        None
    }

    pub fn extend_instance_mods<T>(&mut self, mods: T)
    where
        T: IntoIterator<Item = profile::ModInfo>,
    {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            if let Some(m) = instance.mods.as_mut() {
                m.extend(mods)
            }
        }
    }

    pub fn extend_instance_files<T>(&mut self, files: T)
    where
        T: IntoIterator<Item = (path::PathBuf, profile::FileMetadata)>,
    {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            if let Some(f) = instance.files.as_mut() {
                f.extend(files)
            }
        }
    }
}
