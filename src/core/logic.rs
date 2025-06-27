use std::path::Path;
use std::path::PathBuf;

use iced::widget::combo_box;
use iced::Task;

use crate::app::GothicOrganizer;
use crate::app::Message;
use crate::app::WindowState;
use crate::core::profile;
use crate::error::GothicOrganizerError;
use crate::load_profile;
use crate::load_session;
use crate::save_profile;
use crate::save_session;

pub fn invoke_options_window(app: &mut GothicOrganizer) -> Task<Message> {
    let (id, task) = iced::window::open(iced::window::Settings {
        size: iced::Size {
            width: 400.0,
            height: 400.0,
        },
        icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
        exit_on_close_request: false,
        ..Default::default()
    });

    app.windows.insert(
        Some(id),
        WindowState {
            name: "options".to_owned(),
            closed: false,
        },
    );

    task.then(|_| Task::none())
}

pub fn exit(app: &mut GothicOrganizer, wnd_id: &iced::window::Id) -> Task<Message> {
    write_current_changes(app);
    save_current_session(app);

    if let Some(wnd_state) = app.windows.get_mut(&Some(*wnd_id)) {
        wnd_state.closed = true;
    }

    if app.windows.iter().all(|(_, wnd_state)| wnd_state.closed) {
        iced::exit()
    } else {
        iced::window::get_latest().and_then(iced::window::close)
    }
}

pub fn try_reload_last_session(app: &mut GothicOrganizer) -> Result<(), GothicOrganizerError> {
    let profiles = preload_profiles();
    app.profiles = profiles.clone();
    app.state.profile_choices = combo_box::State::new(profiles.keys().cloned().collect());

    let last_session = load_session!().ok_or(GothicOrganizerError::new("failed to load last session"))?;

    app.theme = last_session.theme;

    let selected_profile_name = last_session
        .selected_profile
        .ok_or(GothicOrganizerError::new("no selected profile"))?
        .clone();

    app.profile_selected = Some(selected_profile_name.clone());

    let selected_profile = profiles
        .get(&selected_profile_name)
        .ok_or(GothicOrganizerError::new(&format!(
            "no profile with name {}",
            &selected_profile_name
        )))?
        .clone();

    let selected_profile_instances = selected_profile.instances.ok_or_else(|| {
        app.files = last_session.cache.unwrap_or_default();
        GothicOrganizerError::new("no instances for selected profile")
    })?;

    app.state.instance_choices = combo_box::State::new(selected_profile_instances.keys().cloned().collect());

    let selected_instance_name = last_session
        .selected_instance
        .ok_or(GothicOrganizerError::new("no selected instance"))?
        .clone();

    app.instance_selected = Some(selected_instance_name.clone());

    let selected_instance = selected_profile_instances
        .get(&selected_instance_name)
        .ok_or(GothicOrganizerError::new(&format!(
            "no instance with name {} for profile {}",
            &selected_instance_name, &selected_profile_name
        )))?
        .clone();

    app.files = selected_instance
        .files
        .ok_or(GothicOrganizerError::new("no files for selected instance"))?;

    Ok(())
}

pub fn init_window(app: &mut GothicOrganizer) -> Task<Message> {
    let (id, task) = iced::window::open(iced::window::Settings {
        size: iced::Size::from(GothicOrganizer::WINDOW_SIZE),
        position: iced::window::Position::Centered,
        icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
        exit_on_close_request: false,
        ..Default::default()
    });

    app.windows.insert(
        Some(id),
        WindowState {
            name: "editor".to_owned(),
            closed: false,
        },
    );

    task.then(|_| Task::done(Message::RefreshFiles))
}

pub fn switch_profile(app: &mut GothicOrganizer, profile_name: &str) -> Task<Message> {
    write_current_changes(app);
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

pub fn write_current_changes(app: &mut GothicOrganizer) {
    let Some(current_profile) = app
        .profiles
        .get_mut(&app.profile_selected.clone().unwrap_or_default())
    else {
        return;
    };

    app.state
        .current_directory_entries
        .iter()
        .for_each(|(path, enabled)| {
            app.files.insert(path.clone(), *enabled);
        });

    if let Some(instances) = current_profile.instances.as_mut()
        && let Some(current_instance) = instances.get_mut(&app.instance_selected.clone().unwrap_or_default())
    {
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
    write_current_changes(app);
    let instance_name = instance_name.to_owned();
    app.instance_selected = Some(instance_name.clone());
    Task::done(Message::RefreshFiles)
}

pub fn browse_game_dir(app: &mut GothicOrganizer, profile_name: &str) -> Task<Message> {
    let profile_name = profile_name.to_owned();
    let Some(path) = rfd::FileDialog::new()
        .set_title(format!("Select {} directory", &profile_name))
        .pick_folder()
    else {
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
            app.files.insert(entry.path().to_path_buf(), true);
        });

    Task::done(Message::RefreshFiles)
}

pub fn refresh_files(app: &mut GothicOrganizer, root: Option<PathBuf>) {
    let Some(current_profile) = app
        .profiles
        .get_mut(&app.profile_selected.clone().unwrap_or_default())
    else {
        return;
    };

    let root_dir = root.unwrap_or_else(|| current_profile.path.clone());
    app.state.current_directory = root_dir.clone();

    let Ok(root_dir_iter) = root_dir.read_dir() else {
        return;
    };

    let profile_dir_entries: Vec<(PathBuf, bool)> = root_dir_iter
        .flatten()
        .map(|entry| (entry.path(), true))
        .collect();

    if let Some(selected_instance) = &app.instance_selected
        && let Some(instances) = &current_profile.instances
        && let Some(current_instance) = instances.get(selected_instance)
    {
        if let Some(instance_files) = &current_instance.files
            && !instance_files.is_empty()
        {
            for (path, enabled) in instance_files.iter() {
                app.files.insert(path.clone(), *enabled);
            }
        }

        app.state.current_directory_entries.clear();

        for (path, _) in &profile_dir_entries {
            if let Some(displayed_state) = app.files.get(path) {
                app.state
                    .current_directory_entries
                    .push((path.clone(), *displayed_state));
            }
        }
    } else {
        app.state.current_directory_entries = profile_dir_entries;
    }

    app.state
        .current_directory_entries
        .sort_unstable_by_key(|(path, _)| !path.is_dir());
}

pub fn save_current_session(app: &mut GothicOrganizer) {
    app.profiles.values().for_each(|p| match save_profile!(p) {
        Ok(_) => {}
        Err(e) => eprintln!("Failed saving profile: {e}"),
    });

    let cache = match app
        .profiles
        .get(&app.profile_selected.clone().unwrap_or_default())
    {
        Some(current_profile) if current_profile.instances.is_some() => None,
        _ => Some(app.files.clone()),
    };

    if let Err(e) = save_session!(
        app.profile_selected.clone(),
        app.instance_selected.clone(),
        cache,
        app.theme.clone()
    ) {
        eprintln!("Failed saving session: {e}");
    }
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

pub fn toggle_state_recursive(app: &mut GothicOrganizer, path: Option<&Path>) {
    if let Some(path) = path
        && let Some(old_state) = app
            .state
            .current_directory_entries
            .iter_mut()
            .find_map(|(p, s)| if p == path { Some(s) } else { None })
    {
        let new_state = !(*old_state);
        *old_state = new_state;
        if path.is_dir() {
            app.files.insert(path.to_path_buf(), new_state);
            app.files.iter_mut().for_each(|(p, s)| {
                if p.starts_with(path) {
                    *s = !(*s);
                }
            })
        }
    } else {
        for (path, state) in app.state.current_directory_entries.iter_mut() {
            let new_state = !(*state);
            *state = new_state;
            if path.is_dir() {
                app.files.insert(path.clone(), new_state);
                app.files.iter_mut().for_each(|(p, s)| {
                    if p.starts_with(path.clone()) {
                        *s = new_state;
                    }
                })
            }
        }
    }
}

pub fn preload_profiles() -> profile::Lookup<String, profile::Profile> {
    profile::Lookup::from(
        crate::core::constants::Profile::into_iter()
            .map(|profile_name| match load_profile!((*profile_name).into()) {
                Some(p) => (profile_name.to_string(), p),
                None => (
                    profile_name.to_string(),
                    profile::Profile::default().with_name((*profile_name).into()),
                ),
            })
            .collect::<Vec<(String, profile::Profile)>>(),
    )
}
