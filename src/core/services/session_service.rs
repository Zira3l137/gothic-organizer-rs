use std::path;

use iced::Task;

use crate::app;
use crate::core;
use crate::error;
use crate::load_config;
use crate::load_profile;
use crate::load_session;
use crate::save_config;
use crate::save_profile;
use crate::save_session;

pub struct SessionService {
    pub profiles: core::lookup::Lookup<String, core::profile::Profile>,
    pub active_profile: Option<String>,
    pub active_instance: Option<String>,
    pub mod_storage_dir: Option<path::PathBuf>,
    pub theme_selected: Option<String>,
    pub files: core::lookup::Lookup<path::PathBuf, core::profile::FileInfo>,
    pub windows: core::lookup::Lookup<Option<iced::window::Id>, app::WindowState>,
}

impl SessionService {
    pub fn new() -> Self {
        let mut service = Self {
            profiles: core::lookup::Lookup::new(),
            active_profile: None,
            active_instance: None,
            mod_storage_dir: None,
            theme_selected: None,
            files: core::lookup::Lookup::new(),
            windows: core::lookup::Lookup::new(),
        };
        service.try_reload_last_session();
        service
    }

    pub fn try_reload_last_session(&mut self) {
        let profiles = Self::preload_profiles();
        self.profiles = profiles.clone();

        if let Some(last_session) = load_session!() {
            if let Some(profile_name) = last_session.selected_profile
                && let Some(profile) = profiles.get(&profile_name)
            {
                if let Some(instances) = &profile.instances {
                    if let Some(instance_name) = last_session.selected_instance
                        && let Some(instance) = instances.get(&instance_name)
                    {
                        self.active_instance = Some(instance_name);
                        self.files = instance.files.clone().unwrap_or_default();
                    }
                } else {
                    self.files = last_session.cache.unwrap_or_default();
                }
                self.active_profile = Some(profile_name);
            }
        }

        if let Some(config) = load_config!() {
            self.theme_selected = Some(config.theme);
            self.mod_storage_dir = Some(config.mod_storage_dir);
        }
    }

    pub fn save_current_session(&self) {
        self.profiles.values().for_each(|p| {
            if let Err(e) = save_profile!(p) {
                log::error!("Failed saving profile: {e}");
            }
        });

        let cache = self
            .active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
            .map_or(Some(self.files.clone()), |profile| {
                profile.instances.is_none().then(|| self.files.clone())
            });

        if let Err(e) = save_session!(
            self.active_profile.clone(),
            self.active_instance.clone(),
            cache
        ) {
            log::error!("Failed saving session: {e}");
        }

        if let Err(e) = save_config!(self.theme_selected.clone(), self.mod_storage_dir.clone()) {
            log::error!("Failed saving config: {e}");
        }
    }

    pub fn exit(&mut self, wnd_id: &iced::window::Id) -> Task<app::Message> {
        self.save_current_session();

        if let Some(wnd_state) = self.windows.get_mut(&Some(*wnd_id)) {
            wnd_state.closed = true;
        }

        if self.windows.iter().all(|(_, wnd_state)| wnd_state.closed) {
            iced::exit()
        } else {
            iced::window::get_latest().and_then(iced::window::close)
        }
    }

    pub fn exit_with_error(&mut self, err: error::SharedError) -> Task<app::Message> {
        log::error!("Error: {err}");
        log::info!("Saving current session and changes");
        self.save_current_session();

        log::info!("Exiting");
        iced::exit()
    }

    pub fn init_window(&mut self) -> Task<app::Message> {
        let (id, task) = iced::window::open(iced::window::Settings {
            size: iced::Size::from(app::GothicOrganizer::WINDOW_SIZE),
            position: iced::window::Position::Centered,
            icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
            exit_on_close_request: false,
            ..Default::default()
        });

        self.windows.insert(
            Some(id),
            app::WindowState {
                name: "editor".to_owned(),
                closed: false,
            },
        );

        task.then(|_| Task::done(app::Message::CurrentDirectoryUpdated))
    }

    pub fn invoke_options_window(&mut self) -> Task<app::Message> {
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

        self.windows.insert(
            Some(id),
            app::WindowState {
                name: "options".to_owned(),
                closed: false,
            },
        );

        task.then(|_| Task::none())
    }

    pub fn preload_profiles() -> core::lookup::Lookup<String, core::profile::Profile> {
        core::constants::Profile::into_iter()
            .map(|profile_name| {
                let name_str = (*profile_name).to_string();
                let profile = load_profile!(&name_str).unwrap_or_else(|| core::profile::Profile::default().with_name(&name_str));
                (name_str, profile)
            })
            .collect()
    }

    pub fn load_default_themes() -> core::lookup::Lookup<String, iced::Theme> {
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
}