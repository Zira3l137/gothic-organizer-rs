use std::path::{Path, PathBuf};

use iced::Task;

use crate::{
    app,
    core::{
        constants,
        lookup::Lookup,
        profile::{FileInfo, Instance, ModInfo},
        utils::{copy_recursive, extract_zip},
    },
    error,
};

pub fn add_mod(app: &mut app::GothicOrganizer, mod_source_path: Option<PathBuf>) -> Task<app::Message> {
    let Some(mod_source_path) = mod_source_path.or_else(|| {
        rfd::FileDialog::new()
            .set_title("Select a zip archive with mod files")
            .add_filter("Zip archive", &["zip"])
            .pick_file()
    }) else {
        return Task::none();
    };

    let mod_path = match move_mod_to_storage(app, &mod_source_path) {
        Ok(path) => path,
        Err(e) => {
            log::warn!("Failed to move mod to storage: {e}");
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
            .collect::<Lookup<PathBuf, FileInfo>>();

        let new_mod_info = ModInfo::default()
            .with_enabled(true)
            .with_name(&mod_name)
            .with_path(&mod_path)
            .with_files(mod_files);

        log::info!(
            "Adding \"{mod_name}\" with {} files",
            new_mod_info.files.len()
        );
        apply_mod_files(instance, &new_mod_info, &profile.path);
        instance
            .mods
            .get_or_insert_with(Vec::new)
            .push(new_mod_info);

        return Task::done(app::Message::RefreshFiles);
    }

    Task::none()
}

pub fn remove_mod(app: &mut app::GothicOrganizer, mod_name: String) -> Task<app::Message> {
    toggle_mod(app, mod_name.clone(), false);
    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(instance_name) = app.instance_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(instance_name)
        && let Some(mods) = instance.mods.as_mut()
        && let Some(mod_info) = mods.iter().find(|info| info.name == mod_name)
    {
        if let Err(e) = std::fs::remove_dir_all(&mod_info.path) {
            return Task::done(app::Message::ReturnError(error::SharedError::new(e)));
        };
        mods.retain(|info| info.name != mod_name);
        instance
            .overwrites
            .as_mut()
            .and_then(|overwrites| overwrites.remove(&mod_name));

        return Task::done(app::Message::RefreshFiles);
    }
    Task::none()
}

pub fn toggle_mod(app: &mut app::GothicOrganizer, mod_name: String, enabled: bool) {
    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(instance_name) = app.instance_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(instance_name)
    {
        let mut mods = instance.mods.clone().unwrap_or_default();
        if let Some(mod_info) = mods.iter_mut().find(|info| info.name == mod_name) {
            if mod_info.enabled == enabled {
                return;
            }

            if enabled {
                log::info!("Enabling \"{mod_name}\"");
                apply_mod_files(instance, mod_info, &profile.path);
            } else {
                log::info!("Disabling \"{mod_name}\"");
                unapply_mod_files(instance, mod_info, &profile.path);
            }
            mod_info.enabled = enabled;
            instance.mods = Some(mods);
        }
    }
}

fn apply_mod_files(instance: &mut Instance, mod_info: &ModInfo, profile_path: &Path) {
    let instance_files = instance.files.get_or_insert_with(Lookup::new);
    mod_info.files.iter().for_each(|(path, info)| {
        if let Ok(relative_path) = path.strip_prefix(&mod_info.path) {
            let dst_path = profile_path.join(relative_path);
            if let Some(existing_file) = instance_files.insert(dst_path.clone(), info.clone().with_target_path(&dst_path)) {
                instance
                    .overwrites
                    .get_or_insert_with(Lookup::new)
                    .access
                    .entry(mod_info.name.clone())
                    .or_default()
                    .insert(dst_path, existing_file);
            }
        }
    });
}

fn unapply_mod_files(instance: &mut Instance, mod_info: &ModInfo, profile_path: &Path) {
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

pub fn load_mods(app: &mut app::GothicOrganizer) -> Task<app::Message> {
    if let Some(profile_name) = app.profile_selected.as_ref()
        && let Some(instance_name) = app.instance_selected.as_ref()
        && let Some(profile) = app.profiles.get_mut(profile_name)
    {
        let base_files = ignore::WalkBuilder::new(&profile.path)
            .ignore(false)
            .build()
            .filter_map(Result::ok)
            .map(|entry| {
                (
                    entry.path().to_path_buf(),
                    FileInfo::default()
                        .with_source_path(entry.path())
                        .with_enabled(true),
                )
            })
            .collect::<Lookup<PathBuf, FileInfo>>();

        if let Some(instances) = profile.instances.as_mut()
            && let Some(instance) = instances.get_mut(instance_name)
        {
            instance.files = Some(base_files);
            instance.overwrites = None;
            let mods = instance.mods.clone().unwrap_or_default();
            for mod_info in mods.iter().filter(|m| m.enabled) {
                apply_mod_files(instance, mod_info, &profile.path);
            }
        }
    } else {
        log::trace!("No mods to load");
    }

    Task::done(app::Message::RefreshFiles)
}

pub fn is_valid_mod_source(mod_path: &Path) -> bool {
    mod_path.exists() && (mod_path.is_dir() || mod_path.extension().and_then(|e| e.to_str()) == Some("zip"))
}

pub fn move_mod_to_storage(app: &app::GothicOrganizer, mod_path: &Path) -> Result<PathBuf, error::GothicOrganizerError> {
    let storage_dir = app.mod_storage_dir.clone().unwrap_or_else(|| {
        constants::default_mod_storage_dir().unwrap_or_else(|e| {
            log::warn!("Failed to get default mod storage dir: {e}");
            PathBuf::from("mods")
        })
    });

    let profile_name = app
        .profile_selected
        .as_ref()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "No profile selected"))?;

    let instance_name = app
        .instance_selected
        .as_ref()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "No instance selected"))?;

    let storage_dir = storage_dir.join(profile_name).join(instance_name);
    log::trace!("Mod storage dir: {}", storage_dir.display());

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

