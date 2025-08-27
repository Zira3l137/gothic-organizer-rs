use iced::widget::combo_box;

use crate::config;
use crate::core::services;

pub mod handlers;
pub mod message;
pub mod state;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub session: services::session::SessionService,
    pub state: state::ApplicationState,
}

impl GothicOrganizer {
    pub const WINDOW_TITLE: &str = "Gothic Organizer";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, iced::Task<message::Message>) {
        let mut session = services::session::SessionService::new();
        let mut state = state::ApplicationState::default();

        Self::initialize_state(&mut session, &mut state);
        let app = Self { session, state };
        (app, iced::Task::done(message::Message::Window(message::WindowMessage::Initialize)))
    }

    fn initialize_state(
        session: &mut services::session::SessionService,
        state: &mut state::ApplicationState,
    ) {
        state.profile.profile_choices =
            combo_box::State::new(session.profile_names.clone().unwrap_or_default());

        state.profile.instance_choices =
            combo_box::State::new(session.instance_names.clone().unwrap_or_default());

        state.ui.themes = services::session::SessionService::load_default_themes();

        state.settings.theme_choices =
            combo_box::State::new(state.ui.themes.iter().map(|(_, t)| t.to_string()).collect());

        state.settings.renderer_choices = combo_box::State::new(
            config::RendererBackend::into_iter().cloned().collect::<Vec<_>>(),
        );

        state.settings.zspy_level_field =
            session.active_zspy_config.get_or_insert_default().verbosity.into();
    }

    pub fn update(&mut self, message: message::Message) -> iced::Task<message::Message> {
        match message {
            message::Message::Profile(msg) => {
                handlers::handle_profile_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::Mod(msg) => {
                handlers::handle_mod_message(&mut self.session, &mut self.state, msg)
            }

            message::Message::UI(msg) => {
                handlers::handle_ui_message(&mut self.session, &mut self.state, msg)
            }

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
                    log::warn!("Theme {theme} not found, defaulting to dark");
                }
                iced::Theme::Dark
            })
    }

    pub fn view(&self, id: iced::window::Id) -> iced::Element<message::Message> {
        if let Some((_, wnd_state)) =
            self.session.windows.iter().find(|(wnd_id, _)| **wnd_id == Some(id))
        {
            match wnd_state.name.as_str() {
                "options" => crate::gui::options::options_view(self),
                "overwrites" => crate::gui::overwrites::overwrites_view(self),
                _ => crate::gui::editor::editor_view(self),
            }
        } else {
            iced::widget::container(iced::widget::text("Window not found")).into()
        }
    }
}
