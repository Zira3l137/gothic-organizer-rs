use std::path::Path;
use std::path::PathBuf;

use iced::widget::combo_box;
use iced::Task;

use crate::app::GothicOrganizer;
use crate::app::Message;
use crate::app::WindowState;
use crate::core::constants;
use crate::core::profile;
use crate::error::GothicOrganizerError;
use crate::load_profile;
use crate::load_session;
use crate::save_profile;
use crate::save_session;

pub fn invoke_options_window(app: &mut GothicOrganizer) -> Task<Message> {
    let (id, task) = iced::window::open(iced::window::Settings {
        position: iced::window::Position::Centered,
        level: iced::window::Level::AlwaysOnTop,
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
        .for_each(|(path, info)| {
            app.files.insert(path.clone(), info.clone());
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

pub fn load_files(app: &mut GothicOrganizer, root: Option<PathBuf>) {
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

    let profile_dir_entries: Vec<(PathBuf, profile::FileInfo)> = root_dir_iter
        .flatten()
        .map(|entry| {
            (
                entry.path(),
                profile::FileInfo::default()
                    .with_source_path(&entry.path())
                    .with_enabled(true),
            )
        })
        .collect();

    if let Some(selected_instance) = &app.instance_selected
        && let Some(instances) = &current_profile.instances
        && let Some(current_instance) = instances.get(selected_instance)
    {
        if let Some(instance_files) = &current_instance.files
            && !instance_files.is_empty()
        {
            for (path, info) in instance_files.iter() {
                app.files.insert(path.clone(), info.clone());
            }
        }

        app.state.current_directory_entries.clear();

        for (path, _) in &profile_dir_entries {
            if let Some(displayed_state) = app.files.get(path) {
                app.state
                    .current_directory_entries
                    .push((path.clone(), displayed_state.clone()));
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
        app.theme.clone().map(|t| t.to_string())
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

// FIXME: This is a mess
pub fn toggle_state_recursive(app: &mut GothicOrganizer, path: Option<&Path>) {
    if let Some(path) = path
        && let Some(old_state) = app
            .state
            .current_directory_entries
            .iter_mut()
            .find_map(|(p, s)| if p == path { Some(s) } else { None })
    {
        let new_state = !(old_state.enabled);
        old_state.enabled = new_state;
        if path.is_dir() {
            app.files.insert(path.to_path_buf(), old_state.clone());
            app.files.iter_mut().for_each(|(p, s)| {
                if p.starts_with(path) {
                    s.enabled = !(s.enabled);
                }
            })
        }
    } else {
        for (path, state) in app.state.current_directory_entries.iter_mut() {
            let new_state = !(state.enabled);
            state.enabled = new_state;
            if path.is_dir() {
                app.files.insert(path.clone(), state.clone());
                app.files.iter_mut().for_each(|(p, s)| {
                    if p.starts_with(path.clone()) {
                        s.enabled = !(s.enabled);
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

pub fn is_valid_mod_source(mod_path: &Path) -> bool {
    (mod_path.is_dir() || mod_path.extension().and_then(|e| e.to_str()) == Some("zip")) && mod_path.exists()
}

pub fn load_default_themes() -> profile::Lookup<String, iced::Theme> {
    profile::Lookup::from(vec![
        (iced::Theme::Light.to_string(), iced::Theme::Light),
        (iced::Theme::Dark.to_string(), iced::Theme::Dark),
        (iced::Theme::Dracula.to_string(), iced::Theme::Dracula),
        (iced::Theme::Nord.to_string(), iced::Theme::Nord),
        (
            iced::Theme::SolarizedLight.to_string(),
            iced::Theme::SolarizedLight,
        ),
        (
            iced::Theme::SolarizedDark.to_string(),
            iced::Theme::SolarizedDark,
        ),
        (
            iced::Theme::GruvboxLight.to_string(),
            iced::Theme::GruvboxLight,
        ),
        (
            iced::Theme::GruvboxDark.to_string(),
            iced::Theme::GruvboxDark,
        ),
        (
            iced::Theme::CatppuccinLatte.to_string(),
            iced::Theme::CatppuccinLatte,
        ),
        (
            iced::Theme::CatppuccinFrappe.to_string(),
            iced::Theme::CatppuccinFrappe,
        ),
        (
            iced::Theme::CatppuccinMacchiato.to_string(),
            iced::Theme::CatppuccinMacchiato,
        ),
        (
            iced::Theme::CatppuccinMocha.to_string(),
            iced::Theme::CatppuccinMocha,
        ),
        (iced::Theme::TokyoNight.to_string(), iced::Theme::TokyoNight),
        (
            iced::Theme::TokyoNightStorm.to_string(),
            iced::Theme::TokyoNightStorm,
        ),
        (
            iced::Theme::TokyoNightLight.to_string(),
            iced::Theme::TokyoNightLight,
        ),
        (
            iced::Theme::KanagawaWave.to_string(),
            iced::Theme::KanagawaWave,
        ),
        (
            iced::Theme::KanagawaDragon.to_string(),
            iced::Theme::KanagawaDragon,
        ),
        (
            iced::Theme::KanagawaLotus.to_string(),
            iced::Theme::KanagawaLotus,
        ),
        (iced::Theme::Moonfly.to_string(), iced::Theme::Moonfly),
        (iced::Theme::Nightfly.to_string(), iced::Theme::Nightfly),
        (iced::Theme::Oxocarbon.to_string(), iced::Theme::Oxocarbon),
        (iced::Theme::Ferra.to_string(), iced::Theme::Ferra),
    ])
}

// FIXME: Zip archive handling not implemented yet
pub fn move_mod_to_storage(app: &mut GothicOrganizer, mod_path: &Path) -> Result<PathBuf, std::io::Error> {
    if !is_valid_mod_source(mod_path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid mod source",
        ));
    }

    let storage_dir = app
        .mods_storage_dir
        .clone()
        .unwrap_or(constants::mod_storage_dir());

    let mod_name = mod_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to get mod name",
        ))?;

    // FIXME: This will fail implement directory copying when it contains files
    std::fs::copy(mod_path, storage_dir.join(&mod_name))?;

    Ok(storage_dir.join(mod_name))
}

pub fn add_mod(
    app: &mut GothicOrganizer,
    profile_name: Option<String>,
    instance_name: Option<String>,
    mod_source_path: Option<PathBuf>,
) -> Task<Message> {
    let Some(mod_source_path) = mod_source_path.or(rfd::FileDialog::new()
        .set_title("Select mod directory or zip archive")
        .pick_folder())
    else {
        return Task::none();
    };

    let mod_path = match move_mod_to_storage(app, &mod_source_path) {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to move mod to storage: {e}");
            return Task::none();
        }
    };

    if let Some(profile_name) = profile_name.or(app.profile_selected.clone())
        && let Some(instance_name) = instance_name.or(app.instance_selected.clone())
        && let Some(profile) = app.profiles.get_mut(&profile_name)
        && let Some(instances) = profile.instances.as_mut()
        && let Some(instance) = instances.get_mut(&instance_name)
        && let Some(mods) = instance.mods.as_mut()
    {
        if !is_valid_mod_source(&mod_path) {
            return Task::none();
        }

        // FIXME: Zip archive handling not implemented yet
        if mod_path.extension().and_then(|e| e.to_str()) == Some("zip") {
            return Task::none();
        }

        let mod_name = mod_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or(format!("Unknown#{}", mods.len() + 1));

        let file_info = |path: &Path| {
            profile::FileInfo::default()
                .with_enabled(true)
                .with_source_path(path)
                .with_parent_name(mod_name.clone())
        };

        let mod_files = ignore::WalkBuilder::new(mod_path.clone())
            .ignore(false)
            .build()
            .filter_map(|e| {
                e.map(|e| (e.path().to_path_buf(), file_info(e.path())))
                    .ok()
            })
            .collect::<profile::Lookup<PathBuf, profile::FileInfo>>();

        let new_mod_info = profile::ModInfo::default()
            .with_enabled(true)
            .with_name(&mod_name)
            .with_path(&mod_path)
            .with_files(mod_files);

        mods.push(new_mod_info);
        return Task::none();
    }

    Task::none()
}

// TODO: implement mod loading from instance
//
// pub fn load_mods(app: &mut GothicOrganizer, profile_name: Option<String>, instance_name: Option<String>) -> Task<Message> {
//     let Some(profile_name) = profile_name.or(app.profile_selected.clone()) else {
//         return Task::none();
//     };
//
//     let Some(instance_name) = instance_name.or(app.instance_selected.clone()) else {
//         return Task::none();
//     };
//
//     if let Some(profile) = app.profiles.get_mut(&profile_name)
//         && let Some(instances) = profile.instances.as_mut()
//         && let Some(instance) = instances.get_mut(&instance_name)
//         && let Some(files) = instance.files.as_mut()
//         && let Some(mods) = instance.mods.as_mut()
//     {
//         mods.iter().for_each(|mod_info| {
//
//         })
//     }
// }
