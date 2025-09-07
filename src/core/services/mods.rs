use std::path;
use std::path::Path;
use std::path::PathBuf;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::core::constants::APP_NAME;
use crate::core::profile::Conflicts;
use crate::core::profile::FileMetadata;
use crate::core::profile::Lookup;
use crate::error;
use crate::error::ErrorContext;

pub struct ModService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

impl<'a> ModService<'a> {
    pub fn new(session: &'a mut session::ApplicationSession, state: &'a mut state::ApplicationState) -> Self {
        Self { session, state }
    }

    pub fn add_mod(&mut self, mod_path: Option<path::PathBuf>) -> Task<message::Message> {
        let Some(mod_path) = mod_path.or_else(|| {
            rfd::FileDialog::new()
                .set_title("Select a zip archive with mod files")
                .add_filter("Zip archive", &["zip"])
                .pick_file()
        }) else {
            tracing::warn!("No mod file selected");
            return Task::none();
        };

        match self.try_add_mod(&mod_path) {
            Ok(()) => Task::done(message::UiMessage::ReloadDirEntries.into()),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn remove_mod(&mut self, mod_index: usize) -> Task<message::Message> {
        match self.try_remove_mod(mod_index) {
            Ok(()) => Task::done(message::UiMessage::ReloadDirEntries.into()),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn toggle_mod(&mut self, mod_index: usize, enabled: bool) -> Task<message::Message> {
        match self.try_toggle_mod(mod_index, enabled) {
            Ok(()) => Task::done(message::UiMessage::ReloadDirEntries.into()),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    pub fn reload_mods(&mut self) -> Task<message::Message> {
        match self.try_reload_mods() {
            Ok(()) => Task::done(message::UiMessage::ReloadDirEntries.into()),
            Err(err) => Task::done(message::ErrorMessage::Handle(err).into()),
        }
    }

    fn try_add_mod(&mut self, mod_path: &Path) -> Result<(), ErrorContext> {
        self.validate_context("Add")?;
        Self::validate_mod(mod_path)?;

        let mod_storage_dir = self.get_mod_storage_dir();
        let mod_name = Self::get_mod_name(mod_path)?;
        tracing::info!("Installing mod \"{}\"", mod_name);

        let mod_dst_path = mod_storage_dir.join(&mod_name);
        Self::install_mod(mod_path, &mod_dst_path)?;

        let mod_info = self.get_mod_info(&mod_dst_path, &mod_name)?;
        let active_profile_name = self.session.active_profile.clone().unwrap();
        let active_instance_name = self.session.active_instance.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(&active_profile_name).unwrap();
        let active_profile_path = active_profile.path.clone();
        let active_instance =
            active_profile.instances.as_mut().unwrap().get_mut(&active_instance_name).unwrap();

        Self::apply_mod_files(
            active_instance.files.get_or_insert_default(),
            active_instance.conflicts.get_or_insert_default(),
            &active_profile_path,
            &mod_info,
        );
        active_instance.mods.get_or_insert_default().push(mod_info.clone());

        Ok(())
    }

    fn try_remove_mod(&mut self, index: usize) -> Result<(), ErrorContext> {
        self.validate_context("Remove")?;
        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();
        let active_instance_name = &self.session.active_instance.clone().unwrap();
        let active_instance =
            active_profile.instances.as_mut().unwrap().get_mut(active_instance_name).unwrap();
        let active_instance_files = active_instance.files.get_or_insert_default();
        let instance_conflicts = active_instance.conflicts.get_or_insert_default();

        let removed_mod_info = active_instance.mods.as_mut().unwrap().remove(index);
        Self::undo_mod_files(
            active_instance_files,
            instance_conflicts,
            &active_profile.path,
            &removed_mod_info,
        );

        let mut errors: usize = removed_mod_info.files.iter().fold(0, |mut errors, (path, _)| {
            let remove = if path.is_dir() { std::fs::remove_dir_all } else { std::fs::remove_file };
            if remove(path).is_err() {
                errors += 1;
            }
            errors
        });

        if std::fs::remove_dir_all(removed_mod_info.path).is_err() {
            errors += 1;
        }

        if errors > 0 {
            return Err(ErrorContext::builder()
                .error(error::Error::new("Failed to remove mod files", "Mods Service", "Remove"))
                .suggested_action(
                    "Check if the mod storage directory is readable or permissions are set correctly.",
                )
                .build());
        }

        Ok(())
    }

    fn try_toggle_mod(&mut self, mod_index: usize, enabled: bool) -> Result<(), ErrorContext> {
        self.validate_context("Toggle")?;
        let active_profile_name = &self.session.active_profile.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(active_profile_name).unwrap();
        let active_instance_name = &self.session.active_instance.clone().unwrap();
        let active_instance =
            active_profile.instances.as_mut().unwrap().get_mut(active_instance_name).unwrap();
        let active_instance_mods = active_instance.mods.as_mut().unwrap();
        let profile_path = active_profile.path.clone();
        let mod_info = active_instance_mods.get_mut(mod_index).unwrap();

        tracing::info!("{} mod \"{}\"", mod_info.name, if enabled { "Enabling" } else { "Disabling" });
        mod_info.enabled = enabled;
        let action = if enabled { Self::apply_mod_files } else { Self::undo_mod_files };
        action(
            active_instance.files.get_or_insert_default(),
            active_instance.conflicts.get_or_insert_default(),
            &profile_path,
            mod_info,
        );

        Ok(())
    }

    fn try_reload_mods(&mut self) -> Result<(), ErrorContext> {
        self.validate_context("Reload")?;
        let active_profile_name = self.session.active_profile.clone().unwrap();
        let active_instance_name = self.session.active_instance.clone().unwrap();
        let active_profile = self.state.profile.profiles.get_mut(&active_profile_name).unwrap();
        let active_instance =
            active_profile.instances.as_mut().unwrap().get_mut(&active_instance_name).unwrap();
        let active_instance_mods = active_instance.mods.as_mut().unwrap();
        let profile_path = active_profile.path.clone();

        for mod_info in active_instance_mods.iter().filter(|m| m.enabled) {
            Self::apply_mod_files(
                active_instance.files.get_or_insert_default(),
                active_instance.conflicts.get_or_insert_default(),
                &profile_path,
                mod_info,
            );
        }

        Ok(())
    }

    fn undo_mod_files(
        instance_files: &mut Lookup<PathBuf, FileMetadata>,
        instance_conflicts: &mut Conflicts,
        profile_path: &path::Path,
        mod_info: &core::profile::ModInfo,
    ) {
        mod_info.files.iter().for_each(|(path, _)| {
            let Ok(relative_path) = path.strip_prefix(&mod_info.path) else { return };
            let dst_path = profile_path.join(relative_path);
            let conflict_list = instance_conflicts.entries.entry(dst_path.clone()).or_default();

            let Some(removed_file_info) = instance_files.remove(&dst_path) else {
                return;
            };

            conflict_list.retain(|f| f != &removed_file_info);
            let Some(original_file) = conflict_list.pop() else {
                return;
            };

            if conflict_list.is_empty() {
                instance_conflicts.entries.remove(&dst_path);
            }

            tracing::info!("Restoring original {}", dst_path.display());
            instance_files.insert(dst_path, original_file);
        });
    }

    fn apply_mod_files(
        instance_files: &mut Lookup<PathBuf, FileMetadata>,
        instance_conflicts: &mut Conflicts,
        profile_path: &path::Path,
        mod_info: &core::profile::ModInfo,
    ) {
        mod_info.files.iter().for_each(|(path, mod_file_info)| {
            let Ok(relative_path) = path.strip_prefix(&mod_info.path) else { return };
            let dst_path = profile_path.join(relative_path);
            let target_file_info = mod_file_info.clone().with_target_path(&dst_path);

            let Some(existing_file_info) = instance_files.insert(dst_path.clone(), target_file_info.clone())
            else {
                return;
            };

            tracing::warn!("{} already exists, overwriting", dst_path.display());
            let conflict_list = instance_conflicts.entries.entry(dst_path.clone()).or_default();
            if !conflict_list.contains(&existing_file_info) {
                conflict_list.push(existing_file_info);
            }
            conflict_list.push(target_file_info);
        });
    }

    fn get_mod_info(&self, mod_path: &Path, mod_name: &str) -> Result<core::profile::ModInfo, ErrorContext> {
        let get_file_info = |path: &path::Path| {
            core::profile::FileMetadata::default()
                .with_enabled(true)
                .with_source_path(path)
                .with_parent_name(mod_name.to_string())
        };

        let mod_files = ignore::WalkBuilder::new(mod_path)
            .ignore(false)
            .build()
            .filter_map(|entry| match entry {
                Ok(e) if e.path() != mod_path => Some((e.path().to_path_buf(), get_file_info(e.path()))),
                _ => None,
            })
            .collect::<Lookup<path::PathBuf, core::profile::FileMetadata>>();

        Ok(core::profile::ModInfo::default()
            .with_enabled(true)
            .with_name(mod_name)
            .with_path(mod_path)
            .with_files(mod_files))
    }

    fn get_mod_name(mod_path: &Path) -> Result<String, ErrorContext> {
        let mut mod_name =
            mod_path.file_name().map(|name| name.to_string_lossy().to_string()).ok_or_else(|| {
                tracing::error!("Failed to get mod name");
                ErrorContext::builder()
                    .error(error::Error::new("Failed to get mod name", "Mods Service", "Add"))
                    .suggested_action(
                        "Select a valid mod file. It should be either a directory or a zip archive.",
                    )
                    .build()
            })?;

        if mod_name.contains(".") {
            mod_name = mod_name.split_once(".").unwrap().0.to_string();
        }

        Ok(mod_name)
    }

    fn get_mod_storage_dir(&self) -> PathBuf {
        let profile_name = self.session.active_profile.clone().unwrap();
        let instance_name = self.session.active_instance.clone().unwrap();
        match self.session.mod_storage_dir.as_ref() {
            Some(dir) => {
                std::path::absolute(dir.join(APP_NAME).join(profile_name).join("mods").join(instance_name))
                    .unwrap()
            }
            None => match self.session.custom_user_data_path.as_ref() {
                Some(custom_path) => std::path::absolute(
                    custom_path.join(APP_NAME).join(profile_name).join("mods").join(instance_name),
                )
                .unwrap(),
                None => {
                    let default_data_path = crate::core::constants::local_app_data_path().join(APP_NAME);
                    std::path::absolute(default_data_path.join(profile_name).join("mods").join(instance_name))
                        .unwrap()
                }
            },
        }
    }

    fn install_mod(src_mod_path: &Path, dst_mod_path: &Path) -> Result<(), ErrorContext> {
        std::fs::create_dir_all(dst_mod_path).map_err(|e| {
            ErrorContext::builder()
                .error(error::Error::new(e.to_string(), "Mods Service", "Add"))
                .suggested_action(
                    "Check if the mod storage directory is writable or permissions are set correctly.",
                )
                .build()
        })?;

        if src_mod_path.is_dir() {
            tracing::info!("Copying mod files to {}", dst_mod_path.display());
            core::utils::copy_recursive(src_mod_path, dst_mod_path)
        } else {
            tracing::info!("Extracting mod archive to {}", dst_mod_path.display());
            core::utils::extract_zip(src_mod_path, dst_mod_path)
        }
    }

    fn validate_mod(mod_path: &path::Path) -> Result<(), ErrorContext> {
        if !Self::is_valid_mod_source(mod_path) {
            return Err(ErrorContext::builder()
                .error(error::Error::new("Invalid mod file", "Mods Service", "Add"))
                .suggested_action(
                    "Select a valid mod file. It should be either a directory or a zip archive.",
                )
                .build());
        };

        Ok(())
    }

    fn validate_context(&self, operation: &str) -> Result<(), ErrorContext> {
        if self.session.active_profile.is_none() {
            Err(ErrorContext::builder()
                .error(error::Error::new("No active profile", "Mods Service", operation))
                .suggested_action("Select a profile and try again")
                .build())
        } else if self.session.active_instance.is_none() {
            return Err(ErrorContext::builder()
                .error(error::Error::new("No active instance", "Mods Service", operation))
                .suggested_action("Select an instance and try again")
                .build());
        } else {
            Ok(())
        }
    }

    fn is_valid_mod_source(mod_path: &path::Path) -> bool {
        mod_path.exists()
            && (mod_path.is_dir() || mod_path.extension().and_then(|e| e.to_str()) == Some("zip"))
    }
}
