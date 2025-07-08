use std::path;

use iced::Task;

use crate::app;
use crate::core;
use crate::core::services::Service;
use crate::error;

pub struct ModService<'a> {
    session: &'a mut core::services::session_service::SessionService,
}

impl super::Service for ModService<'_> {
    fn context(&mut self) -> Result<super::context::Context, crate::error::GothicOrganizerError> {
        let profile = self
            .session
            .active_profile
            .as_mut()
            .and_then(|p| self.session.profiles.get_mut(&p.clone()))
            .ok_or_else(|| crate::error::GothicOrganizerError::Other("No active profile".into()))?;

        let instance_name = self
            .session
            .active_instance
            .as_ref()
            .ok_or_else(|| crate::error::GothicOrganizerError::Other("No active instance".into()))?;

        Ok(super::context::Context::new(profile, instance_name))
    }
}

impl<'a> ModService<'a> {
    pub fn new(session: &'a mut core::services::session_service::SessionService) -> Self {
        Self { session }
    }

    pub fn add_mod(&mut self, mod_source_path: Option<path::PathBuf>) -> Task<app::Message> {
        let Some(mod_source_path) = mod_source_path.or_else(|| {
            rfd::FileDialog::new()
                .set_title("Select a zip archive with mod files")
                .add_filter("Zip archive", &["zip"])
                .pick_file()
        }) else {
            return Task::none();
        };

        let mod_path = match self.move_mod_to_storage(&mod_source_path) {
            Ok(path) => path,
            Err(e) => {
                log::warn!("Failed to move mod to storage: {e}");
                return Task::none();
            }
        };

        if Self::is_valid_mod_source(&mod_path) {
            let Ok(mut context) = self.context() else {
                return Task::none();
            };

            let profile_path = context.active_profile.path.clone();

            let mod_name = mod_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("Unknown#{}", chrono::Local::now().timestamp_millis()));

            let file_info = |path: &path::Path| {
                core::profile::FileInfo::default()
                    .with_enabled(true)
                    .with_source_path(path)
                    .with_parent_name(mod_name.clone())
            };

            let mod_files = ignore::WalkBuilder::new(mod_path.clone())
                .ignore(false)
                .build()
                .filter_map(|e| {
                    e.ok()
                        .map(|e| (e.path().to_path_buf(), file_info(e.path())))
                })
                .collect::<core::lookup::Lookup<path::PathBuf, core::profile::FileInfo>>();

            let new_mod_info = core::profile::ModInfo::default()
                .with_enabled(true)
                .with_name(&mod_name)
                .with_path(&mod_path)
                .with_files(mod_files);

            Self::apply_mod_files(context.instance_mut(None), &new_mod_info, &profile_path);
            context
                .instance_mut(None)
                .mods
                .get_or_insert_default()
                .push(new_mod_info.clone());

            return Task::done(app::Message::CurrentDirectoryUpdated);
        }

        Task::none()
    }

    pub fn remove_mod(&mut self, mod_name: String) -> Task<app::Message> {
        self.toggle_mod(mod_name.clone(), false);
        let Ok(mut context) = self.context() else {
            return Task::none();
        };

        let mods = context.instance_mods_mut();

        if let Some(mods) = mods
            && let Some(mod_info) = mods.iter().find(|info| info.name == mod_name)
        {
            if let Err(e) = std::fs::remove_dir_all(&mod_info.path) {
                return Task::done(app::Message::ErrorReturned(error::SharedError::new(e)));
            };

            mods.retain(|info| info.name != mod_name);

            context
                .instance_mut(None)
                .overwrites
                .as_mut()
                .and_then(|overwrites| overwrites.remove(&mod_name));

            return Task::done(app::Message::CurrentDirectoryUpdated);
        }
        Task::none()
    }

    pub fn toggle_mod(&mut self, mod_name: String, enabled: bool) {
        let Ok(mut context) = self.context() else {
            return;
        };
        let profile_path = context.active_profile.path.clone();

        let mods = context.instance_mods().cloned();
        if let Some(mut mods) = mods
            && let Some(mod_info) = mods.iter_mut().find(|info| info.name == mod_name)
        {
            if mod_info.enabled == enabled {
                return;
            }

            if enabled {
                log::info!("Enabling \"{mod_name}\"");
                Self::apply_mod_files(context.instance_mut(None), mod_info, &profile_path);
            } else {
                log::info!("Disabling \"{mod_name}\"");
                Self::unapply_mod_files(context.instance_mut(None), mod_info, &profile_path);
            }

            mod_info.enabled = enabled;
            context.instance_mut(None).mods = Some(mods);
        }
    }

    fn apply_mod_files(instance: &mut core::profile::Instance, mod_info: &core::profile::ModInfo, profile_path: &path::Path) {
        let instance_files = instance.files.get_or_insert_with(core::lookup::Lookup::new);
        mod_info.files.iter().for_each(|(path, info)| {
            if let Ok(relative_path) = path.strip_prefix(&mod_info.path) {
                let dst_path = profile_path.join(relative_path);
                if let Some(existing_file) = instance_files.insert(dst_path.clone(), info.clone().with_target_path(&dst_path)) {
                    instance
                        .overwrites
                        .get_or_insert_with(core::lookup::Lookup::new)
                        .access
                        .entry(mod_info.name.clone())
                        .or_default()
                        .insert(dst_path, existing_file);
                }
            }
        });
    }

    fn unapply_mod_files(instance: &mut core::profile::Instance, mod_info: &core::profile::ModInfo, profile_path: &path::Path) {
        let Some(instance_files) = instance.files.as_mut() else {
            return;
        };
        mod_info.files.iter().for_each(|(path, _)| {
            if let Ok(relative_path) = path.strip_prefix(&mod_info.path) {
                let dst_path = profile_path.join(relative_path);
                instance_files.remove(&dst_path);
                if let Some(mod_overwrites) = instance
                    .overwrites
                    .as_mut()
                    .and_then(|o| o.get_mut(&mod_info.name))
                    && let Some(original_file) = mod_overwrites.remove(&dst_path)
                {
                    instance_files.insert(dst_path, original_file);
                }
            }
        });
    }

    pub fn reload_mods(&mut self) -> Task<app::Message> {
        let Ok(mut context) = self.context() else {
            return Task::none();
        };

        let profile_path = context.active_profile.path.clone();
        let current_instance_mods = context.instance_mods().cloned();

        if let Some(mods) = current_instance_mods {
            context.instance_mut(None).overwrites = None;
            for mod_info in mods.iter().filter(|m| m.enabled) {
                Self::apply_mod_files(context.instance_mut(None), mod_info, &profile_path);
            }
        }

        Task::done(app::Message::CurrentDirectoryUpdated)
    }

    pub fn is_valid_mod_source(mod_path: &path::Path) -> bool {
        mod_path.exists() && (mod_path.is_dir() || mod_path.extension().and_then(|e| e.to_str()) == Some("zip"))
    }

    pub fn move_mod_to_storage(&mut self, mod_path: &path::Path) -> Result<path::PathBuf, error::GothicOrganizerError> {
        let storage_dir = self.session.mod_storage_dir.clone().unwrap_or_else(|| {
            core::constants::default_mod_storage_dir().unwrap_or_else(|e| {
                log::warn!("Failed to get default mod storage dir: {e}");
                path::PathBuf::from("mods")
            })
        });

        let context = self.context()?;
        let profile_name = context.active_profile.name.clone();
        let instance_name = context.active_instance_name.clone();

        let storage_dir = storage_dir.join(profile_name).join(instance_name);

        let mod_name = mod_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.strip_suffix(".zip").unwrap_or(s))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to get mod name"))?;

        let dst_dir = storage_dir.join(mod_name);
        if dst_dir.exists() {
            return Err(error::GothicOrganizerError::from(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Mod already exists",
            )));
        }

        std::fs::create_dir_all(&dst_dir)?;

        if mod_path.is_dir() {
            core::utils::copy_recursive(mod_path, &dst_dir)?;
        } else {
            core::utils::extract_zip(mod_path, &dst_dir)?;
        }

        Ok(dst_dir)
    }
}
