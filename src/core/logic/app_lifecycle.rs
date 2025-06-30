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
    app.state.profile_choices = combo_box::State::new(profiles.keys().cloned().collect());
    app.profiles = profiles.clone();

    let last_session = load_session!().ok_or_else(|| GothicOrganizerError::new("Failed to load last session"))?;
    app.theme = last_session.theme;

    if let Some(profile_name) = last_session.selected_profile
        && let Some(profile) = app.profiles.get(&profile_name)
        && let Some(instance_name) = last_session.selected_instance
        && let Some(instances) = &profile.instances
        && let Some(instance) = instances.get(&instance_name)
    {
        app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect());
        app.files = instance.files.clone().unwrap_or_default();
        app.instance_selected = Some(instance_name);
        app.profile_selected = Some(profile_name);
    } else {
        app.files = last_session.cache.unwrap_or_default();
    }

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

pub fn save_current_session(app: &GothicOrganizer) {
    app.profiles.values().for_each(|p| {
        if let Err(e) = save_profile!(p) {
            eprintln!("Failed saving profile: {e}");
        }
    });

    let cache = app
        .profile_selected
        .as_ref()
        .and_then(|name| app.profiles.get(name))
        .map_or(Some(app.files.clone()), |profile| {
            profile.instances.is_none().then(|| app.files.clone())
        });

    if let Err(e) = save_session!(
        app.profile_selected.clone(),
        app.instance_selected.clone(),
        cache,
        app.theme.clone()
    ) {
        eprintln!("Failed saving session: {e}");
    }
}

pub fn load_default_themes() -> profile::Lookup<String, iced::Theme> {
    [
        (iced::Theme::Light, "Light"),
        (iced::Theme::Dark, "Dark"),
        (iced::Theme::Dracula, "Dracula"),
        (iced::Theme::Nord, "Nord"),
        (iced::Theme::SolarizedLight, "SolarizedLight"),
        (iced::Theme::SolarizedDark, "SolarizedDark"),
        (iced::Theme::GruvboxLight, "GruvboxLight"),
        (iced::Theme::GruvboxDark, "GruvboxDark"),
        (iced::Theme::CatppuccinLatte, "CatppuccinLatte"),
        (iced::Theme::CatppuccinFrappe, "CatppuccinFrappe"),
        (iced::Theme::CatppuccinMacchiato, "CatppuccinMacchiato"),
        (iced::Theme::CatppuccinMocha, "CatppuccinMocha"),
        (iced::Theme::TokyoNight, "TokyoNight"),
        (iced::Theme::TokyoNightStorm, "TokyoNightStorm"),
        (iced::Theme::TokyoNightLight, "TokyoNightLight"),
        (iced::Theme::KanagawaWave, "KanagawaWave"),
        (iced::Theme::KanagawaDragon, "KanagawaDragon"),
        (iced::Theme::KanagawaLotus, "KanagawaLotus"),
        (iced::Theme::Moonfly, "Moonfly"),
        (iced::Theme::Nightfly, "Nightfly"),
        (iced::Theme::Oxocarbon, "Oxocarbon"),
        (iced::Theme::Ferra, "Ferra"),
    ]
    .into_iter()
    .map(|(theme, name)| (name.to_string(), theme))
    .collect()
}
