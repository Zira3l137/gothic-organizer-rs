use std::path::PathBuf;

use iced::widget::combo_box;
use iced::Task;

use crate::app;
use crate::core::lookup::Lookup;
use crate::core::profile;
use crate::error;
use crate::load_profile;

pub fn switch_profile(app: &mut app::GothicOrganizer, profile_name: &str) -> Task<app::Message> {
    update_instance_from_cache(app);

    if let Some(next_profile) = app.profiles.get(profile_name) {
        log::trace!("Loading profile {profile_name}");
        app.profile_selected = Some(profile_name.to_owned());
        app.instance_selected = None;

        let instances = next_profile
            .instances
            .as_ref()
            .map(|i| i.keys().cloned().collect())
            .unwrap_or_default();
        app.state.instance_choices = combo_box::State::new(instances);

        if !next_profile.path.as_os_str().is_empty() {
            return Task::done(app::Message::RefreshFiles);
        }
    }

    Task::none()
}

pub fn update_instance_from_cache(app: &mut app::GothicOrganizer) {
    if let Some(profile_name) = app.profile_selected.clone()
        && let Some(profile) = app.profiles.get_mut(&profile_name)
        && let Some(instance_name) = app.instance_selected.clone()
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(&instance_name)
    {
        app.files
            .extend(app.state.current_directory_entries.iter().cloned());

        instance.files = Some(app.files.clone());
    }
}

pub fn add_instance_for_profile(app: &mut app::GothicOrganizer, profile_name: &str) -> Task<app::Message> {
    if let Some(profile) = app.profiles.get_mut(profile_name) {
        let instance_name = app
            .state
            .instance_input
            .clone()
            .unwrap_or_else(|| format!("{profile_name}_{}", chrono::Local::now().timestamp()));

        let new_instance = profile::Instance::default()
            .with_name(&instance_name)
            .with_files(Some(app.files.clone()));

        let instances = profile.instances.get_or_insert_with(Default::default);
        if instances.contains_key(&instance_name) {
            return Task::none();
        }

        instances.insert(instance_name, new_instance);
        app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect());

        return Task::done(app::Message::RefreshFiles);
    }

    Task::none()
}

pub fn remove_instance_from_profile(app: &mut app::GothicOrganizer, profile_name: &str) {
    if let Some(profile) = app.profiles.get_mut(profile_name)
        && let Some(selected_instance_name) = app.instance_selected.clone()
        && let Some(instances) = profile.instances.as_mut()
    {
        instances.remove(&selected_instance_name);
        app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect());
        app.instance_selected = None;
        app.state.instance_input = None;
        if instances.is_empty() {
            profile.instances = None;
        }
    }
}

pub fn select_instance(app: &mut app::GothicOrganizer, instance_name: &str) -> Task<app::Message> {
    update_instance_from_cache(app);
    app.instance_selected = Some(instance_name.to_owned());
    log::trace!("Loading files for instance {instance_name}");
    Task::done(app::Message::LoadMods)
}

pub fn set_game_dir(app: &mut app::GothicOrganizer, profile_name: Option<String>, path: Option<PathBuf>) -> Task<app::Message> {
    if let Some(profile_name) = profile_name.or_else(|| app.profile_selected.clone())
        && let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select {} directory", &profile_name))
                .pick_folder()
        })
        && path.is_dir()
        && let Some(profile) = app.profiles.get_mut(&profile_name)
    {
        log::trace!("Setting {} directory to {}", profile_name, path.display());
        profile.path = path.clone();
        app.state.current_directory = path.clone();

        app.files.extend(
            ignore::WalkBuilder::new(path)
                .ignore(false)
                .build()
                .filter_map(Result::ok)
                .map(|entry| {
                    (
                        entry.path().to_path_buf(),
                        profile::FileInfo::default()
                            .with_source_path(entry.path())
                            .with_enabled(true),
                    )
                }),
        );

        return Task::done(app::Message::RefreshFiles);
    }

    Task::none()
}

pub fn set_mods_dir(app: &mut app::GothicOrganizer, profile_name: Option<String>, path: Option<PathBuf>) -> Task<app::Message> {
    if let Some(profile_name) = profile_name.or_else(|| app.profile_selected.clone())
        && let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select {} directory", &profile_name))
                .pick_folder()
        })
        && path.is_dir()
        && app.profiles.get(&profile_name).is_some()
    {
        log::trace!(
            "Setting {profile_name} mods directory to {}",
            path.display()
        );
        app.mod_storage_dir = Some(path.clone());

        if let Err(err) = std::fs::create_dir_all(&path) {
            return Task::done(app::Message::ReturnError(error::SharedError::new(err)));
        } else {
            return Task::done(app::Message::RefreshFiles);
        }
    }

    Task::none()
}

pub fn preload_profiles() -> Lookup<String, profile::Profile> {
    crate::core::constants::Profile::into_iter()
        .map(|profile_name| {
            let name_str = (*profile_name).to_string();
            let profile = load_profile!(&name_str).unwrap_or_else(|| profile::Profile::default().with_name(&name_str));
            (name_str, profile)
        })
        .collect()
}
