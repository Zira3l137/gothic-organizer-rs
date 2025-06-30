use iced::{widget::combo_box, Task};

use crate::{
    app::{GothicOrganizer, Message, WindowState},
    core::{
        logic::profile_management::{preload_profiles, write_changes_to_instance},
        profile::{self},
    },
    error::GothicOrganizerError,
    load_session, save_profile, save_session,
};

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
    write_changes_to_instance(app);
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
