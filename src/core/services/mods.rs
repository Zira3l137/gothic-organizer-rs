use std::path;
use std::path::Path;
use std::path::PathBuf;

use iced::Task;

use crate::app::message;
use crate::app::session;
use crate::app::state;
use crate::core;
use crate::core::profile::Lookup;
use crate::core::services::ApplicationContext;
use crate::error;

pub struct ModService<'a> {
    session: &'a mut session::ApplicationSession,
    state: &'a mut state::ApplicationState,
}

crate::impl_app_context!(ModService);

impl<'a> ModService<'a> {
    pub fn new(session: &'a mut session::ApplicationSession, state: &'a mut state::ApplicationState) -> Self {
        Self { session, state }
    }

    /// Adds a mod to the current instance. Mod files are copied to the instance's mod storage
    /// directory. If the mod storage directory doesn't exist, it is created. Finally, reloads
    /// UI files.
    ///
    /// Emits an error message in following cases:
    ///     1. If no instance is active.
    ///     2. If failed to get context.
    ///     3. If mod with the same name already exists.
    ///     4. If mod file is invalid.
    ///     5. If failed to move mod to storage.
    pub fn add_mod(&mut self, mod_source_path: Option<path::PathBuf>) -> Task<message::Message> {
        let mut current_mod_storage_dir = self.session.mod_storage_dir.clone().unwrap_or_default();

        let mut context = match self.context() {
            Ok(ctx) => ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        };

        if !current_mod_storage_dir.exists() {
            let default_path =
                crate::core::constants::local_profiles_path().join(&context.active_profile.name).join("mods");
            if !default_path.exists() {
                if let Err(e) = std::fs::create_dir_all(&default_path) {
                    tracing::error!("Failed to create default mod storage directory: {e}");
                    return Task::done(message::ErrorMessage::Handle(e.into()).into());
                }
                current_mod_storage_dir = default_path;
            }
        }

        let profile_name = context.active_profile.name.clone();
        let instance_name = context.active_instance_name.clone();
        let profile_path = context.active_profile.path.clone();

        let Some(instance) = context.instance_mut(None) else {
            tracing::error!("No active instance");
            return Task::done(
                message::ErrorMessage::Handle(
                    error::ErrorContext::builder()
                        .error(error::Error::new("No active instance", "Mods Service", "Add"))
                        .suggested_action("Select an instance and try again")
                        .build(),
                )
                .into(),
            );
        };

        let Some(mod_source_path) = mod_source_path.or_else(|| {
            rfd::FileDialog::new()
                .set_title("Select a zip archive with mod files")
                .add_filter("Zip archive", &["zip"])
                .pick_file()
        }) else {
            tracing::warn!("No mod file selected");
            return Task::none();
        };

        if let Some(msg) = Self::validate_mod(&mod_source_path) {
            tracing::error!("Failed to add mod: {}", mod_source_path.display());
            return Task::done(msg.into());
        }

        let mod_path = match Self::move_mod_to_storage(
            &mod_source_path,
            &current_mod_storage_dir,
            &profile_name,
            &instance_name,
        ) {
            Ok(path) => path,
            Err(e) => {
                tracing::error!("Failed to move mod to storage: {e}");
                return Task::done(message::ErrorMessage::Handle(error::ErrorContext::from(e)).into());
            }
        };

        let mod_name = mod_path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap();
        tracing::info!("Adding mod \"{mod_name}\"");

        let get_file_info = |path: &path::Path| {
            core::profile::FileMetadata::default()
                .with_enabled(true)
                .with_source_path(path)
                .with_parent_name(mod_name.clone())
        };

        let mod_files = ignore::WalkBuilder::new(mod_path.clone())
            .ignore(false)
            .build()
            .filter_map(|entry| match entry {
                Ok(e) if e.path() != mod_path => Some((e.path().to_path_buf(), get_file_info(e.path()))),
                _ => None,
            })
            .collect::<Lookup<path::PathBuf, core::profile::FileMetadata>>();

        let new_mod_info = core::profile::ModInfo::default()
            .with_enabled(true)
            .with_name(&mod_name)
            .with_path(&mod_path)
            .with_files(mod_files);

        Self::toggle_mod_files(instance, &profile_path, &new_mod_info, true);
        instance.mods.get_or_insert_default().push(new_mod_info.clone());

        Task::done(message::UiMessage::ReloadDirEntries.into())
    }

    /// Removes a mod from the instance and its storage directory. Also removes it from the
    /// overwrites list if it exists. Finally, reloads UI files.
    ///
    /// Emits an error message in following cases:
    ///    1. Could not get context.
    ///    2. Failed to remove mod directory.
    pub fn remove_mod(&mut self, mod_name: String) -> Task<message::Message> {
        let mut tasks: Vec<Task<message::Message>> = vec![self.toggle_mod(mod_name.clone(), false)];
        let mut context = match self.context() {
            Ok(ctx) => ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                tasks.push(Task::done(message::ErrorMessage::Handle(err.into()).into()));
                return Task::batch(tasks);
            }
        };

        let mods = context.instance_mods_mut();
        if let Some(mods) = mods
            && let Some(mod_info) = mods.iter().find(|info| info.name == mod_name)
        {
            tracing::info!("Removing \"{mod_name}\"");
            if let Err(e) = std::fs::remove_dir_all(&mod_info.path) {
                tracing::error!("Failed to remove mod directory: {e}");
                tasks.push(Task::done(message::ErrorMessage::Handle(error::ErrorContext::from(e)).into()));
                return Task::batch(tasks);
            };

            mods.retain(|info| info.name != mod_name);
            return Task::batch(tasks);
        }

        tracing::warn!("Mod \"{mod_name}\" not found");
        Task::batch(tasks)
    }

    /// Enables or disables a mod. Adding or removing it from the instance's mods list, as well as
    /// its files and overwrites. Finally, reloads UI files.
    ///
    /// Emits an error message if failed to get context.
    pub fn toggle_mod(&mut self, mod_name: String, enabled: bool) -> Task<message::Message> {
        let mut context = match self.context() {
            Ok(ctx) => ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        };

        let profile_path = context.active_profile.path.clone();
        let mods = context.instance_mods().cloned();

        let Some(instance) = context.instance_mut(None) else { return Task::none() };
        if let Some(mut mods) = mods
            && let Some(mod_info) = mods.iter_mut().find(|info| info.name == mod_name)
        {
            tracing::info!("{} mod \"{mod_name}\"", if enabled { "Enabling" } else { "Disabling" });
            Self::toggle_mod_files(instance, &profile_path, mod_info, enabled);

            mod_info.enabled = enabled;
            context.set_instance_mods(mods);

            tracing::info!("Reloading UI files");
            return Task::done(message::Message::UI(message::UiMessage::ReloadDirEntries));
        }

        tracing::warn!("Mod \"{mod_name}\" not found");
        Task::none()
    }

    /// Adds or removes mod files from the instance's files list, as well as its overwrites.
    pub fn toggle_mod_files(
        instance: &mut core::profile::Instance,
        profile_path: &path::Path,
        mod_info: &core::profile::ModInfo,
        enable: bool,
    ) {
        let instance_files = instance.files.get_or_insert_default();
        mod_info.files.iter().for_each(|(path, mod_file_info)| {
            let instance_overwrites = instance.overridden_files.get_or_insert_default();
            let Ok(relative_path) = path.strip_prefix(&mod_info.path) else { return };
            let dst_path = profile_path.join(relative_path);

            if enable {
                let target_file_info = mod_file_info.clone().with_target_path(&dst_path);
                let Some(existing_file_info) =
                    instance_files.insert(dst_path.clone(), target_file_info.clone())
                else {
                    return;
                };

                tracing::warn!("{} already exists, overwriting", dst_path.display());
                let override_list = instance_overwrites.entries.entry(dst_path.clone()).or_default();
                if !override_list.contains(&existing_file_info) {
                    override_list.push(existing_file_info);
                }
                override_list.push(target_file_info);
            } else {
                let override_list = instance_overwrites.entries.entry(dst_path.clone()).or_default();

                let Some(removed_file_info) = instance_files.remove(&dst_path) else {
                    tracing::warn!("{} not found in instance", mod_info.path.join(relative_path).display());
                    return;
                };

                override_list.retain(|f| f != &removed_file_info);
                let Some(original_file) = override_list.pop() else {
                    return;
                };

                if override_list.is_empty() {
                    instance_overwrites.entries.remove(&dst_path);
                }

                tracing::info!("Restoring original {}", dst_path.display());
                instance_files.insert(dst_path, original_file);
            }
        });
    }

    /// Reloads mods by clearing the instance's overwrites list and reloading the UI files.
    ///
    /// Emits an error message if failed to get context or no active instance.
    pub fn reload_mods(&mut self) -> Task<message::Message> {
        let mut context = match self.context() {
            Ok(ctx) => ctx,
            Err(err) => {
                tracing::error!("Failed to get context: {err}");
                return Task::done(message::ErrorMessage::Handle(err.into()).into());
            }
        };

        let profile_path = context.active_profile.path.clone();
        let current_instance_mods = context.instance_mods().cloned();

        let Some(mods) = current_instance_mods else {
            tracing::warn!("No mods found");
            return Task::none();
        };

        context.clear_instance_overwrites();
        let Some(instance) = context.instance_mut(None) else {
            tracing::error!("No active instance");
            return Task::done(
                message::ErrorMessage::Handle(
                    error::ErrorContext::builder()
                        .error(error::Error::new("No active instance", "Mods Service", "Reload"))
                        .suggested_action("Select an instance and try again")
                        .build(),
                )
                .into(),
            );
        };

        tracing::info!("Reloading mods");
        for mod_info in mods.iter().filter(|m| m.enabled) {
            Self::toggle_mod_files(instance, &profile_path, mod_info, true);
        }

        Task::none()
    }

    /// Checks if a mod file is valid. Returns `false` if mod file doesn't exist or is not a
    /// directory or a zip archive.
    pub fn is_valid_mod_source(mod_path: &path::Path) -> bool {
        mod_path.exists()
            && (mod_path.is_dir() || mod_path.extension().and_then(|e| e.to_str()) == Some("zip"))
    }

    /// Moves a mod file to the storage directory. Returns the path to the mod in the storage
    /// directory. Storage directory is created if it doesn't exist. If it wasn't provided by user
    /// via UI, default location is used.
    ///
    /// Returns error if mod with the same name already exists
    pub fn move_mod_to_storage(
        mod_path: &path::Path,
        mod_storage_dir: &Path,
        profile_name: &str,
        instance_name: &str,
    ) -> Result<path::PathBuf, error::Error> {
        let storage_dir = Self::get_current_storage_dir(mod_storage_dir, profile_name, instance_name);
        let mod_name = mod_path
            .file_name()
            .and_then(|name| name.to_str().map(|s| s.strip_suffix(".zip").unwrap_or(s)))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to get mod name"))?;

        let dst_dir = storage_dir.join(mod_name);
        if dst_dir.exists() {
            return Err(error::Error::new("Mod already exists", "Mods Service", "Move Mod to Storage"));
        }

        std::fs::create_dir_all(&dst_dir)?;

        if mod_path.is_dir() {
            tracing::info!("Copying mod files to {}", dst_dir.display());
            core::utils::copy_recursive(mod_path, &dst_dir)?;
        } else {
            tracing::info!("Extracting mod archive to {}", dst_dir.display());
            core::utils::extract_zip(mod_path, &dst_dir)?;
        }

        Ok(dst_dir)
    }

    /// Returns current instance's mod storage directory for the current profile.
    pub fn get_current_storage_dir(
        mod_storage_dir: &Path,
        profile_name: &str,
        instance_name: &str,
    ) -> PathBuf {
        let mut base_path = mod_storage_dir.to_path_buf();
        if !base_path.components().any(|c| c.as_os_str() == profile_name) {
            base_path = base_path.join(profile_name);
        }
        base_path.join(instance_name)
    }

    /// Checks if mod source is valid and returns error message if not.
    pub fn validate_mod(mod_path: &path::Path) -> Option<message::ErrorMessage> {
        if !Self::is_valid_mod_source(mod_path) {
            return Some(message::ErrorMessage::Handle(
                error::ErrorContext::builder()
                    .error(error::Error::new("Invalid mod file", "Mods Service", "Add"))
                    .suggested_action(
                        "Select a valid mod file. It should be either a directory or a zip archive.",
                    )
                    .build(),
            ));
        };

        None
    }
}
