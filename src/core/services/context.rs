#![allow(dead_code)]

use std::path;

use crate::core::lookup;
use crate::core::profile;

type Overwrites = lookup::Lookup<String, lookup::Lookup<path::PathBuf, profile::FileMetadata>>;
type Mods = Vec<profile::ModInfo>;
type Instances = lookup::Lookup<String, profile::Instance>;
type InstanceFiles = lookup::Lookup<path::PathBuf, profile::FileMetadata>;

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

    pub fn instance_overwrites(&self) -> Option<&Overwrites> {
        self.instance(Some(&self.active_instance_name))?.overwrites.as_ref()
    }

    pub fn instance_overwrites_mut(&mut self) -> Option<&mut Overwrites> {
        self.instance_mut(Some(&self.active_instance_name.clone()))?.overwrites.as_mut()
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

    pub fn clear_instance_overwrites(&mut self) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.overwrites = None;
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

    pub fn set_instance_overwrites(&mut self, overwrites: Overwrites) {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            instance.overwrites = Some(overwrites);
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

    pub fn extend_instance_overwrites<T>(&mut self, overwrites: T)
    where
        T: IntoIterator<
            Item = (
                String,
                crate::core::lookup::Lookup<path::PathBuf, crate::core::profile::FileMetadata>,
            ),
        >,
    {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            if let Some(o) = instance.overwrites.as_mut() {
                o.extend(overwrites)
            }
        }
    }

    pub fn extend_instance_mods<T>(&mut self, mods: T)
    where
        T: IntoIterator<Item = crate::core::profile::ModInfo>,
    {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            if let Some(m) = instance.mods.as_mut() {
                m.extend(mods)
            }
        }
    }

    pub fn extend_instance_files<T>(&mut self, files: T)
    where
        T: IntoIterator<Item = (path::PathBuf, crate::core::profile::FileMetadata)>,
    {
        if let Some(instance) = self.instance_mut(Some(&self.active_instance_name.clone())) {
            if let Some(f) = instance.files.as_mut() {
                f.extend(files)
            }
        }
    }
}
