use iced::Task;

use crate::app::message;
use crate::app::state;
use crate::core::lookup;
use crate::save_app_preferences;
use crate::save_app_session;
use crate::save_profile;

pub struct SessionService<'a> {
    session: &'a mut crate::app::session::ApplicationSession,
    state: &'a mut crate::app::state::ApplicationState,
}

impl<'a> SessionService<'a> {
    pub fn new(
        session: &'a mut crate::app::session::ApplicationSession,
        state: &'a mut crate::app::state::ApplicationState,
    ) -> Self {
        Self { session, state }
    }

    pub fn save_current_session(&self) {
        self.state.profile.profiles.values().for_each(|p| {
            if let Err(e) = save_profile!(p) {
                log::error!("Failed saving profile: {e}");
            }
        });

        if let Err(e) = save_app_session!(self.session) {
            log::error!("Failed saving session: {e}");
        }

        if let Err(e) = save_app_preferences!(
            self.session.theme_selected.clone(),
            self.session.mod_storage_dir.clone()
        ) {
            log::error!("Failed saving config: {e}");
        }
    }

    pub fn close_window(&mut self, wnd_id: &iced::window::Id) -> Task<message::Message> {
        if let Some(wnd_state) = self.state.windows.window_states.get_mut(&Some(*wnd_id)) {
            wnd_state.is_closed = true;
        }

        iced::Task::chain(
            iced::window::get_latest().and_then(iced::window::close),
            Task::done(message::SystemMessage::ExitApplication.into()),
        )
    }

    pub fn init_window(&mut self) -> Task<message::Message> {
        let (id, task) = iced::window::open(iced::window::Settings {
            size: iced::Size::from(crate::app::GothicOrganizer::WINDOW_SIZE),
            position: iced::window::Position::Centered,
            icon: iced::window::icon::from_file("./resources/icon.ico").ok(),
            exit_on_close_request: false,
            ..Default::default()
        });

        self.state
            .windows
            .window_states
            .insert(Some(id), state::WindowInfo { name: "editor".to_owned(), is_closed: false });

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

        self.state
            .windows
            .window_states
            .insert(Some(id), state::WindowInfo { name: "options".to_owned(), is_closed: false });

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

        self.state.windows.window_states.insert(
            Some(id),
            state::WindowInfo { name: "overwrites".to_owned(), is_closed: false },
        );

        task.then(|_| Task::none())
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
