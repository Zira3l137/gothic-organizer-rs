use std::path::{Path, PathBuf};

use iced::Task;

use crate::{
    app::{GothicOrganizer, Message},
    core::{
        constants,
        profile::{self, FileInfo, ModInfo},
    },
    error::GothicOrganizerError,
};

use super::super::utils::{copy_recursive, extract_zip};

pub fn add_mod(app: &mut GothicOrganizer, mod_source_path: Option<PathBuf>) -> Task<Message> {
    let Some(mod_source_path) = mod_source_path.or_else(|| {
        rfd::FileDialog::new()
            .set_title("Select a zip archive with mod files")
            .add_filter("Zip archive", &["zip"])
            .pick_file()
    }) else {
        return Task::none();
    };

    log::trace!("Attempting to add mod from: {}", mod_source_path.display());

    let mod_path = match move_mod_to_storage(app, &mod_source_path) {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to move mod to storage: {e}");
            return Task::none();
        }
    };

    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(instance_name) = app.instance_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(instance_name)
        && is_valid_mod_source(&mod_path)
    {
        let mod_name = mod_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("Unknown#{}", chrono::Local::now().timestamp_millis()));
        log::trace!("Assigned name: {mod_name}");

        let file_info = |path: &Path| {
            FileInfo::default()
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
            .collect::<profile::Lookup<PathBuf, FileInfo>>();

        let new_mod_info = ModInfo::default()
            .with_enabled(true)
            .with_name(&mod_name)
            .with_path(&mod_path)
            .with_files(mod_files);

        log::trace!("Adding mod to instance");
        instance
            .mods
            .get_or_insert_with(Vec::new)
            .push(new_mod_info);

        return Task::done(Message::LoadMods);
    }

    Task::none()
}

pub fn remove_mod(app: &mut GothicOrganizer, mod_name: String) -> Task<Message> {
    let storage_dir = match app.mods_storage_dir.as_ref() {
        Some(dir) => dir.clone(),
        None => match constants::default_mod_storage_dir() {
            Ok(dir) => dir,
            Err(e) => {
                log::error!("Failed to get default mod storage dir: {e}");
                return Task::none();
            }
        },
    };

    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(instance_name) = app.instance_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(instance_name)
        && let Some(mods) = instance.mods.as_mut()
    {
        mods.retain(|m| m.name != mod_name);

        if mods.is_empty() {
            instance.mods = None;
        }

        let mod_dir = storage_dir.join(&mod_name);
        if mod_dir.exists() {
            log::trace!("Removing mod directory {}", mod_dir.display());
            if let Err(e) = std::fs::remove_dir_all(&mod_dir) {
                log::error!("Failed to remove mod directory: {e}");
            }
        }

        return Task::chain(
            Task::done(Message::RefreshFiles),
            Task::done(Message::LoadMods),
        );
    }

    Task::none()
}

pub fn load_mods(app: &mut GothicOrganizer) -> Task<Message> {
    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(instance_name) = app.instance_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(instance_name)
        && let Some(instance_files) = instance.files.as_mut()
        && let Some(instance_mods) = instance.mods.as_mut()
        && !instance_mods.is_empty()
    {
        instance_mods.iter().for_each(|mod_info| {
            log::trace!("Loading mod {}", mod_info.name);
            mod_info.files.iter().for_each(|(path, info)| {
                if let Ok(relative_path) = path.strip_prefix(&mod_info.path) {
                    let dst_path = profile.path.join(relative_path);

                    log::trace!("Inserting file {} to instance files", path.display());
                    if let Some(existing_file) = instance_files.insert(dst_path.clone(), info.clone().with_target_path(&dst_path)) {
                        log::trace!("Overwriting file {}", existing_file.source_path.display());
                        instance
                            .overwrtites
                            .get_or_insert_with(profile::Lookup::new)
                            .insert(path.clone(), existing_file);
                    }
                }
            })
        });
    } else {
        log::trace!("No mods to load");
    }

    Task::done(Message::RefreshFiles)
}

pub fn is_valid_mod_source(mod_path: &Path) -> bool {
    mod_path.exists() && (mod_path.is_dir() || mod_path.extension().and_then(|e| e.to_str()) == Some("zip"))
}

pub fn move_mod_to_storage(app: &GothicOrganizer, mod_path: &Path) -> Result<PathBuf, GothicOrganizerError> {
    let storage_dir = app.mods_storage_dir.clone().unwrap_or_else(|| {
        constants::default_mod_storage_dir().unwrap_or_else(|e| {
            log::error!("Failed to get default mod storage dir: {e}");
            PathBuf::from("mods")
        })
    });
    log::trace!("Mod storage dir: {}", storage_dir.display());

    let mod_name = mod_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.strip_suffix(".zip").unwrap_or(s))
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to get mod name"))?;

    let dst_dir = storage_dir.join(mod_name);

    if dst_dir.exists() {
        return Err(GothicOrganizerError::from(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Mod already exists",
        )));
    }

    log::trace!("Creating mod directory {}", dst_dir.display());
    std::fs::create_dir_all(&dst_dir)?;

    if mod_path.is_dir() {
        log::trace!("Copying mod files");
        copy_recursive(mod_path, &dst_dir)?;
    } else {
        log::trace!("Extracting mod files");
        extract_zip(mod_path, &dst_dir)?;
    }

    Ok(dst_dir)
}
