use std::path::PathBuf;

use iced::{
    widget::combo_box,
    Task,
};

use crate::{
    app::{GothicOrganizer, Message},
    core::profile::{self, Profile},
    load_profile,
};

pub fn switch_profile(app: &mut GothicOrganizer, profile_name: &str) -> Task<Message> {
    write_changes_to_instance(app);
    let next_profile_name = profile_name.to_owned();

    let Some(next_profile) = app.profiles.get(&next_profile_name) else {
        return Task::none();
    };

    app.profile_selected = Some(next_profile_name.clone());
    app.instance_selected = None;

    if let Some(instances) = &next_profile.instances {
        app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect::<Vec<String>>());
    } else {
        app.state.instance_choices = combo_box::State::new(Vec::new());
    };

    if next_profile.path.display().to_string().is_empty() {
        Task::none()
    } else {
        Task::done(Message::RefreshFiles)
    }
}

pub fn write_changes_to_instance(app: &mut GothicOrganizer) {
    let Some(current_profile) = app
        .profiles
        .get_mut(&app.profile_selected.clone().unwrap_or_default())
    else {
        return;
    };

    log::trace!("Fetching current directory changes");
    app.state
        .current_directory_entries
        .iter()
        .for_each(|(path, info)| {
            app.files.insert(path.clone(), info.clone());
        });

    if let Some(instances) = current_profile.instances.as_mut()
        && let Some(current_instance) = instances.get_mut(&app.instance_selected.clone().unwrap_or_default())
    {
        log::trace!(
            "Writing current changes into instance {}",
            current_instance.name
        );
        current_instance.files = Some(app.files.clone());
    }
}

pub fn add_instance_for_profile(app: &mut GothicOrganizer, profile_name: &str) -> Task<Message> {
    let profile_name = profile_name.to_owned();
    let instance_name = get_instance_name(app, &profile_name);
    let new_instance = profile::Instance::default().with_name(&instance_name);

    let Some(current_profile) = app.profiles.get_mut(&profile_name) else {
        return Task::none();
    };

    let Some(instances) = current_profile.instances.as_mut() else {
        let new_instances = profile::Lookup::from(vec![new_instance]);
        app.state.instance_choices = combo_box::State::new(new_instances.keys().cloned().collect::<Vec<String>>());
        current_profile.instances = Some(new_instances);
        return Task::done(Message::RefreshFiles);
    };

    if instances.contains_key(&instance_name) {
        return Task::none();
    }

    instances.insert(instance_name.to_owned(), new_instance.clone());
    app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect::<Vec<String>>());

    Task::done(Message::RefreshFiles)
}

pub fn remove_instance_from_profile(app: &mut GothicOrganizer, profile_name: &str) {
    let profile_name = profile_name.to_owned();
    let selected_instance_name = app.instance_selected.clone().unwrap_or_default();

    if let Some(profile) = app.profiles.get_mut(&profile_name)
        && let Some(instances) = profile.instances.as_mut()
    {
        instances.remove(&selected_instance_name);
        app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect::<Vec<String>>());
        app.instance_selected = None;
        app.state.instance_input = None;
        if instances.is_empty() {
            profile.instances = None;
        }
    }
}

pub fn select_instance(app: &mut GothicOrganizer, instance_name: &str) -> Task<Message> {
    write_changes_to_instance(app);
    let instance_name = instance_name.to_owned();
    app.instance_selected = Some(instance_name.clone());
    write_changes_to_instance(app);
    Task::done(Message::RefreshFiles)
}

pub fn set_game_dir(app: &mut GothicOrganizer, profile_name: Option<String>, path: Option<PathBuf>) -> Task<Message> {
    let Some(profile_name) = profile_name.or(app.profile_selected.clone()) else {
        return Task::none();
    };

    let Some(path) = path.or_else(|| {
        rfd::FileDialog::new()
            .set_title(format!("Select {} directory", &profile_name))
            .pick_folder()
    }) else {
        return Task::none();
    };

    if !path.exists() || !path.is_dir() {
        return Task::none();
    };

    let Some(profile) = app.profiles.get_mut(&profile_name) else {
        return Task::none();
    };

    profile.path = path.clone();
    app.state.current_directory = path.clone();

    ignore::WalkBuilder::new(path)
        .ignore(false)
        .build()
        .filter_map(Result::ok)
        .for_each(|entry| {
            app.files.insert(
                entry.path().to_path_buf(),
                profile::FileInfo::default()
                    .with_source_path(entry.path())
                    .with_enabled(true),
            );
        });

    Task::done(Message::RefreshFiles)
}

pub fn get_instance_name(app: &mut GothicOrganizer, profile_name: &str) -> String {
    app.state.instance_input.clone().unwrap_or_else(|| {
        format!(
            "{}_instance_{}",
            profile_name,
            chrono::Local::now().timestamp()
        )
    })
}

pub fn preload_profiles() -> profile::Lookup<String, Profile> {
    profile::Lookup::from(
        crate::core::constants::Profile::into_iter()
            .map(|profile_name| match load_profile!((*profile_name).into()) {
                Some(p) => (profile_name.to_string(), p),
                None => (
                    profile_name.to_string(),
                    Profile::default().with_name((*profile_name).into()),
                ),
            })
            .collect::<Vec<(String, Profile)>>(),
    )
}