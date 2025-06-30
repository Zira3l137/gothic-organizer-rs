use std::path::PathBuf;

use iced::{widget::combo_box, Task};

use crate::{
    app::{GothicOrganizer, Message},
    core::profile::{self, Profile},
    load_profile,
};

pub fn switch_profile(app: &mut GothicOrganizer, profile_name: &str) -> Task<Message> {
    log::trace!("Writing current changes");
    write_changes_to_instance(app);

    log::trace!("Switching to profile {profile_name}");
    if let Some(next_profile) = app.profiles.get(profile_name) {
        log::trace!("Loading profile {profile_name}");
        app.profile_selected = Some(profile_name.to_owned());
        app.instance_selected = None;

        log::trace!("Updating instance choices");
        let instances = next_profile
            .instances
            .as_ref()
            .map(|i| i.keys().cloned().collect())
            .unwrap_or_default();
        app.state.instance_choices = combo_box::State::new(instances);

        if !next_profile.path.as_os_str().is_empty() {
            log::trace!("Loading profile files");
            return Task::done(Message::RefreshFiles);
        }
    }

    Task::none()
}

pub fn write_changes_to_instance(app: &mut GothicOrganizer) {
    if let Some(profile_name) = app.profile_selected.clone()
        && let Some(profile) = app.profiles.get_mut(&profile_name)
        && let Some(instance_name) = app.instance_selected.clone()
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(&instance_name)
    {
        log::trace!("Fetching current directory changes");
        app.files
            .extend(app.state.current_directory_entries.iter().cloned());

        log::trace!("Writing current changes into instance {}", instance.name);
        instance.files = Some(app.files.clone());
    }
}

pub fn add_instance_for_profile(app: &mut GothicOrganizer, profile_name: &str) -> Task<Message> {
    if let Some(profile) = app.profiles.get_mut(profile_name) {
        let instance_name = app
            .state
            .instance_input
            .clone()
            .unwrap_or_else(|| format!("{profile_name}_{}", chrono::Local::now().timestamp()));

        let new_instance = profile::Instance::default().with_name(&instance_name);

        let instances = profile.instances.get_or_insert_with(Default::default);
        if instances.contains_key(&instance_name) {
            return Task::none();
        }

        instances.insert(instance_name, new_instance);
        app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect());

        return Task::done(Message::RefreshFiles);
    }

    Task::none()
}

pub fn remove_instance_from_profile(app: &mut GothicOrganizer, profile_name: &str) {
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

pub fn select_instance(app: &mut GothicOrganizer, instance_name: &str) -> Task<Message> {
    write_changes_to_instance(app);
    app.instance_selected = Some(instance_name.to_owned());
    Task::done(Message::RefreshFiles)
}

pub fn set_game_dir(app: &mut GothicOrganizer, profile_name: Option<String>, path: Option<PathBuf>) -> Task<Message> {
    if let Some(profile_name) = profile_name.or_else(|| app.profile_selected.clone())
        && let Some(path) = path.or_else(|| {
            rfd::FileDialog::new()
                .set_title(format!("Select {} directory", &profile_name))
                .pick_folder()
        })
        && path.is_dir()
        && let Some(profile) = app.profiles.get_mut(&profile_name)
    {
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

        return Task::done(Message::RefreshFiles);
    }

    Task::none()
}

pub fn preload_profiles() -> profile::Lookup<String, Profile> {
    crate::core::constants::Profile::into_iter()
        .map(|profile_name| {
            let name_str = (*profile_name).to_string();
            let profile = load_profile!(&name_str).unwrap_or_else(|| Profile::default().with_name(&name_str));
            (name_str, profile)
        })
        .collect()
}
