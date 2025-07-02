use iced::widget::combo_box;
use iced::Task;

use crate::app;
use crate::core::logic::profile_management;
use crate::core::lookup::Lookup;
use crate::error;
use crate::load_config;
use crate::load_session;
use crate::save_config;
use crate::save_profile;
use crate::save_session;

pub fn invoke_options_window(app: &mut app::GothicOrganizer) -> Task<app::Message> {
    let (id, task) = iced::window::open(iced::window::Settings {
        position: iced::window::Position::Centered,
        size: iced::Size {
            width: 768.0,
            height: 400.0,
        },
        icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
        exit_on_close_request: false,
        ..Default::default()
    });

    app.windows.insert(
        Some(id),
        app::WindowState {
            name: "options".to_owned(),
            closed: false,
        },
    );

    task.then(|_| Task::none())
}

pub fn exit(app: &mut app::GothicOrganizer, wnd_id: &iced::window::Id) -> Task<app::Message> {
    profile_management::update_instance_from_cache(app);
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

pub fn exit_with_error(app: &mut app::GothicOrganizer, err: error::SharedError) -> Task<app::Message> {
    log::error!("Error: {err}");
    log::info!("Saving current session and changes");
    profile_management::update_instance_from_cache(app);
    save_current_session(app);

    log::info!("Exiting");
    iced::exit()
}

pub fn try_reload_last_session(app: &mut app::GothicOrganizer) -> Result<(), error::GothicOrganizerError> {
    let profiles = profile_management::preload_profiles();
    app.state.profile_choices = combo_box::State::new(profiles.keys().cloned().collect());
    app.profiles = profiles.clone();

    let last_session = load_session!().ok_or_else(|| error::GothicOrganizerError::new("Failed to load last session"))?;

    if let Some(profile_name) = last_session.selected_profile
        && let Some(profile) = profiles.get(&profile_name)
    {
        if let Some(instances) = &profile.instances {
            app.state.instance_choices = combo_box::State::new(instances.keys().cloned().collect());
            if let Some(instance_name) = last_session.selected_instance
                && let Some(instance) = instances.get(&instance_name)
            {
                app.instance_selected = Some(instance_name);
                app.files = instance.files.clone().unwrap_or_default();
            }
        } else {
            app.files = last_session.cache.unwrap_or_default();
        }
        app.profile_selected = Some(profile_name);
    }

    let config = load_config!().ok_or_else(|| error::GothicOrganizerError::new("Failed to load config"))?;
    app.theme = Some(config.theme);
    app.mod_storage_dir = Some(config.mod_storage_dir);

    Ok(())
}

pub fn init_window(app: &mut app::GothicOrganizer) -> Task<app::Message> {
    let (id, task) = iced::window::open(iced::window::Settings {
        size: iced::Size::from(app::GothicOrganizer::WINDOW_SIZE),
        position: iced::window::Position::Centered,
        icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
        exit_on_close_request: false,
        ..Default::default()
    });

    app.windows.insert(
        Some(id),
        app::WindowState {
            name: "editor".to_owned(),
            closed: false,
        },
    );

    task.then(|_| Task::done(app::Message::RefreshFiles))
}

pub fn save_current_session(app: &app::GothicOrganizer) {
    app.profiles.values().for_each(|p| {
        if let Err(e) = save_profile!(p) {
            log::error!("Failed saving profile: {e}");
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
        cache
    ) {
        log::error!("Failed saving session: {e}");
    }

    if let Err(e) = save_config!(app.theme.clone(), app.mod_storage_dir.clone()) {
        log::error!("Failed saving config: {e}");
    }
}

pub fn load_default_themes() -> Lookup<String, iced::Theme> {
    [
        ("Light", iced::Theme::Light),
        ("Dark", iced::Theme::Dark),
        ("Dracula", iced::Theme::Dracula),
        ("Nord", iced::Theme::Nord),
        ("Solarized Light", iced::Theme::SolarizedLight),
        ("Solarized Dark", iced::Theme::SolarizedDark),
        ("Gruvbox Light", iced::Theme::GruvboxLight),
        ("Gruvbox Dark", iced::Theme::GruvboxDark),
        ("Catppuccin Latte", iced::Theme::CatppuccinLatte),
        ("Catppuccin Frappé", iced::Theme::CatppuccinFrappe),
        ("Catppuccin Macchiato", iced::Theme::CatppuccinMacchiato),
        ("Catppuccin Mocha", iced::Theme::CatppuccinMocha),
        ("Tokyo Night", iced::Theme::TokyoNight),
        ("Tokyo Night Storm", iced::Theme::TokyoNightStorm),
        ("Tokyo Night Light", iced::Theme::TokyoNightLight),
        ("Kanagawa Wave", iced::Theme::KanagawaWave),
        ("Kanagawa Dragon", iced::Theme::KanagawaDragon),
        ("Kanagawa Lotus", iced::Theme::KanagawaLotus),
        ("Moonfly", iced::Theme::Moonfly),
        ("Nightfly", iced::Theme::Nightfly),
        ("Oxocarbon", iced::Theme::Oxocarbon),
        ("Ferra", iced::Theme::Ferra),
    ]
    .into_iter()
    .map(|(name, theme)| (name.to_owned(), theme))
    .collect()
}
