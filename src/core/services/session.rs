use std::path;

use iced::Task;

use crate::app::message;
use crate::app::state;
use crate::config;
use crate::core::constants;
use crate::core::lookup;
use crate::core::profile;
use crate::error;
use crate::load_app_preferences;
use crate::load_app_session;
use crate::load_profile;
use crate::lookup;
use crate::save_app_preferences;
use crate::save_app_session;
use crate::save_profile;

#[derive(Debug, Default)]
pub struct SessionService {
    pub profiles: lookup::Lookup<String, profile::Profile>,
    pub profile_names: Option<Vec<String>>,
    pub instance_names: Option<Vec<String>>,
    pub active_profile: Option<String>,
    pub active_instance: Option<String>,
    pub active_renderer_backend: Option<config::RendererBackend>,
    pub active_zspy_config: Option<config::ZspyConfig>,
    pub mod_storage_dir: Option<path::PathBuf>,
    pub theme_selected: Option<String>,
    pub files: lookup::Lookup<path::PathBuf, profile::FileMetadata>,
    pub windows: lookup::Lookup<Option<iced::window::Id>, crate::app::state::WindowState>,
    pub launch_options: Option<config::GameLaunchConfiguration>,
}

impl SessionService {
    pub fn new() -> Self {
        let mut service = Self {
            profiles: lookup::Lookup::new(),
            profile_names: None,
            instance_names: None,
            active_profile: None,
            active_instance: None,
            active_renderer_backend: None,
            active_zspy_config: None,
            mod_storage_dir: None,
            theme_selected: None,
            launch_options: None,
            files: lookup::Lookup::new(),
            windows: lookup::Lookup::new(),
        };
        service.try_reload_last_session();
        service
    }

    pub fn try_reload_last_session(&mut self) {
        let profiles = Self::preload_profiles();
        self.profiles = profiles.clone();
        self.profile_names = Some(profiles.keys().cloned().collect());

        if let Some(last_session) = load_app_session!() {
            if let Some(launch_options) = last_session.game_launch_config {
                self.launch_options = Some(launch_options.clone());
                self.active_renderer_backend = Some(launch_options.game_settings.renderer);
                self.active_zspy_config = Some(launch_options.game_settings.zspy);
            }

            if let Some(profile_name) = last_session.active_profile_name
                && let Some(profile) = profiles.get(&profile_name)
            {
                if let Some(instances) = &profile.instances {
                    self.instance_names = Some(instances.keys().cloned().collect());
                    if let Some(instance_name) = last_session.active_instance_name
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

        if let Some(prefs) = load_app_preferences!() {
            self.theme_selected = Some(prefs.theme_name);
            self.mod_storage_dir = Some(prefs.mod_storage_dir);
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

        if let Err(e) = save_app_session!(
            self.active_profile.clone(),
            self.active_instance.clone(),
            self.launch_options.clone(),
            cache
        ) {
            log::error!("Failed saving session: {e}");
        }

        if let Err(e) =
            save_app_preferences!(self.theme_selected.clone(), self.mod_storage_dir.clone())
        {
            log::error!("Failed saving config: {e}");
        }
    }

    pub fn close_window(&mut self, wnd_id: &iced::window::Id) -> Task<message::Message> {
        if let Some(wnd_state) = self.windows.get_mut(&Some(*wnd_id)) {
            wnd_state.is_closed = true;
        }

        iced::Task::chain(
            iced::window::get_latest().and_then(iced::window::close),
            Task::done(message::SystemMessage::ExitApplication.into()),
        )
    }
    pub fn exit_with_error(&mut self, err: error::SharedError) -> Task<message::Message> {
        log::error!("Error: {err}");
        log::info!("Saving current session and changes");
        self.save_current_session();
        log::info!("Exiting");
        iced::exit()
    }

    pub fn init_window(&mut self) -> Task<message::Message> {
        let (id, task) = iced::window::open(iced::window::Settings {
            size: iced::Size::from(crate::app::GothicOrganizer::WINDOW_SIZE),
            position: iced::window::Position::Centered,
            icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
            exit_on_close_request: false,
            ..Default::default()
        });

        self.windows
            .insert(Some(id), state::WindowState { name: "editor".to_owned(), is_closed: false });

        task.then(|_| Task::done(message::UiMessage::ReloadDirEntries.into()))
    }

    pub fn invoke_options_window(&mut self) -> Task<message::Message> {
        let (id, task) = iced::window::open(iced::window::Settings {
            position: iced::window::Position::Centered,
            size: iced::Size { width: 768.0, height: 460.0 },
            icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
            exit_on_close_request: false,
            ..Default::default()
        });

        self.windows
            .insert(Some(id), state::WindowState { name: "options".to_owned(), is_closed: false });

        task.then(|_| Task::none())
    }

    pub fn invoke_overwrites_window(&mut self) -> Task<message::Message> {
        let (id, task) = iced::window::open(iced::window::Settings {
            position: iced::window::Position::Centered,
            size: iced::Size { width: 460.0, height: 768.0 },
            icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
            exit_on_close_request: false,
            ..Default::default()
        });

        self.windows.insert(
            Some(id),
            state::WindowState { name: "overwrites".to_owned(), is_closed: false },
        );

        task.then(|_| Task::none())
    }

    pub fn preload_profiles() -> lookup::Lookup<String, profile::Profile> {
        constants::Profile::into_iter()
            .map(|profile_name| {
                let name_str = (*profile_name).to_string();
                let profile = load_profile!(&name_str)
                    .unwrap_or_else(|| profile::Profile::default().with_name(&name_str));
                (name_str, profile)
            })
            .collect()
    }

    pub fn toggle_launch_option(&mut self, option: &config::ParserCommand, new_state: bool) {
        if let Some(options) = self.launch_options.as_mut() {
            options.parser_settings.commands.insert(option.clone(), new_state);
        } else {
            self.launch_options = Some(config::GameLaunchConfiguration {
                parser_settings: config::ParserSettings {
                    commands: lookup![(option.clone() => new_state)],
                },
                ..Default::default()
            });
        }
    }

    pub fn load_default_themes() -> lookup::Lookup<String, iced::Theme> {
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
