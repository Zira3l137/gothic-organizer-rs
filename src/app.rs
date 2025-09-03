use iced::widget::combo_box::State;

use crate::load_app_session;

pub mod handlers;
pub mod message;
pub mod session;
pub mod state;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub session: session::ApplicationSession,
    pub state: state::ApplicationState,
}

impl GothicOrganizer {
    pub const WINDOW_TITLE: &str = "Gothic Organizer";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new(user_data_dir: Option<std::path::PathBuf>) -> (Self, iced::Task<message::Message>) {
        let mut state = match user_data_dir.clone() {
            Some(path) => state::ApplicationState::new(path),
            None => state::ApplicationState::default(),
        };

        let mut session = load_app_session!(user_data_dir.as_deref()).unwrap_or_default();
        session.custom_user_data_path = user_data_dir;
        Self::initialize_state(&mut session, &mut state);

        let app = Self { session, state };
        (app, iced::Task::done(message::Message::Window(message::WindowMessage::Initialize)))
    }

    fn initialize_state(session: &mut session::ApplicationSession, state: &mut state::ApplicationState) {
        let active_profile_name = session.active_profile.as_ref();
        let active_profile = active_profile_name.and_then(|n| state.profile.profiles.get(n));
        let instances = active_profile.and_then(|p| p.instances.as_ref());
        let instance_names = instances.map(|i| i.keys().cloned().collect::<Vec<_>>()).unwrap_or_default();
        let renderers = session::RendererBackend::into_iter().cloned().collect::<Vec<_>>();
        let zspy_level = session.active_zspy_config.get_or_insert_default().verbosity;
        let themes = crate::core::helpers::default_themes().map(|pair| pair.0.to_owned()).to_vec();

        state.ui.themes = crate::core::helpers::default_themes()
            .into_iter()
            .map(|(name, theme)| (name.to_owned(), theme))
            .collect();

        state.profile.instance_choices = State::new(instance_names);
        state.settings.theme_choices = State::new(themes);
        state.settings.renderer_choices = State::new(renderers);
        state.settings.zspy_level_field = zspy_level.into();
    }

    pub fn update(&mut self, message: message::Message) -> iced::Task<message::Message> {
        match message {
            message::Message::Profile(msg) => {
                handlers::handle_profile_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::Mod(msg) => {
                handlers::handle_mod_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::UI(msg) => handlers::handle_ui_message(&mut self.session, &mut self.state, msg),

            message::Message::Settings(msg) => {
                handlers::handle_settings_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::Window(msg) => {
                handlers::handle_window_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::System(msg) => {
                handlers::handle_system_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::Error(msg) => {
                handlers::handle_error_message(&mut self.session, &mut self.state, msg)
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<message::Message> {
        iced::event::listen_with(|event, _, id| match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => {
                Some(message::Message::Window(message::WindowMessage::Close(id)))
            }
            iced::Event::Window(iced::window::Event::FileDropped(path)) => {
                Some(message::Message::Mod(message::ModMessage::Add(Some(path))))
            }
            _ => None,
        })
    }

    pub fn theme(&self) -> iced::Theme {
        self.session
            .theme_selected
            .as_ref()
            .and_then(|theme| self.state.ui.themes.get(theme).cloned())
            .unwrap_or_else(|| {
                if let Some(theme) = &self.session.theme_selected {
                    tracing::warn!("Theme {theme} not found, defaulting to dark");
                }
                iced::Theme::Dark
            })
    }

    pub fn view(&self, id: iced::window::Id) -> iced::Element<message::Message> {
        if let Some((_, wnd_state)) = self.state.ui.windows.iter().find(|(wnd_id, _)| **wnd_id == Some(id)) {
            match wnd_state.name.as_str() {
                "options" => crate::gui::options::options_view(self),
                "logs" => crate::gui::logs::logs_view(self),
                _ => crate::gui::editor::editor_view(self),
            }
        } else {
            iced::widget::container(iced::widget::text("Window not found")).into()
        }
    }
}
