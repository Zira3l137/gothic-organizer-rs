use std::path;

use crate::core::lookup;
use crate::core::profile;

#[allow(unused)]
type Overwrites = lookup::Lookup<String, lookup::Lookup<path::PathBuf, profile::FileInfo>>;
type Mods = Vec<profile::ModInfo>;
type InstanceFiles = lookup::Lookup<path::PathBuf, profile::FileInfo>;

#[derive(Debug)]
pub struct Context<'ctx> {
    pub active_profile: &'ctx mut profile::Profile,
    pub active_instance_name: String,
}

impl<'ctx> Context<'ctx> {
    pub fn new(profile: &'ctx mut profile::Profile, instance_name: &str) -> Self {
        Self {
            active_profile: profile,
            active_instance_name: instance_name.to_owned(),
        }
    }

    pub fn instance(&self, instance_name: Option<&str>) -> &profile::Instance {
        self.active_profile
            .instances
            .as_ref()
            .expect("Tried to get instance from profile that has no instances")
            .get(instance_name.unwrap_or_else(|| &self.active_instance_name))
            .expect("Tried to get instance from profile that has no instances")
    }

    pub fn instance_mut(&mut self, instance_name: Option<&str>) -> &mut profile::Instance {
        self.active_profile
            .instances
            .as_mut()
            .expect("Tried to get instance from profile that has no instances")
            .get_mut(instance_name.unwrap_or_else(|| &self.active_instance_name))
            .expect("Tried to get instance from profile that has no instances")
    }

    pub fn instance_mods(&self) -> Option<&Mods> {
        self.instance(Some(&self.active_instance_name))
            .mods
            .as_ref()
    }

    pub fn instance_mods_mut(&mut self) -> Option<&mut Mods> {
        self.instance_mut(Some(&self.active_instance_name.clone()))
            .mods
            .as_mut()
    }

    #[allow(unused)]
    pub fn instance_overwrites(&self) -> Option<&Overwrites> {
        self.instance(Some(&self.active_instance_name))
            .overwrites
            .as_ref()
    }

    #[allow(unused)]
    pub fn instance_overwrites_mut(&mut self) -> Option<&mut Overwrites> {
        self.instance_mut(Some(&self.active_instance_name.clone()))
            .overwrites
            .as_mut()
    }

    pub fn instance_files(&self) -> Option<&InstanceFiles> {
        self.instance(Some(&self.active_instance_name))
            .files
            .as_ref()
    }

    #[allow(unused)]
    pub fn instance_files_mut(&mut self) -> Option<&mut InstanceFiles> {
        self.instance_mut(Some(&self.active_instance_name.clone()))
            .files
            .as_mut()
    }
}
