use std::path::PathBuf;

use iced::widget::combo_box;

use crate::core::lookup;
use crate::core::profile;
use crate::core::services::mod_service;
use crate::core::services::profile_service;
use crate::core::services::session_service;
use crate::core::services::ui_service;
use crate::error;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub session: session_service::SessionService,
    pub state: InnerState,
}

#[derive(Debug, Default)]
pub struct InnerState {
    pub current_directory: PathBuf,
    pub instance_input: String,
    pub profile_directory_input: String,
    pub mods_directory_input: String,
    pub profile_choices: combo_box::State<String>,
    pub theme_choices: combo_box::State<String>,
    pub instance_choices: combo_box::State<String>,
    pub themes: lookup::Lookup<String, iced::Theme>,
    pub current_directory_entries: Vec<(PathBuf, profile::FileInfo)>,
}

#[derive(Debug, Default)]
pub struct WindowState {
    pub name: String,
    pub closed: bool,
}

impl GothicOrganizer {
    pub const WINDOW_TITLE: &str = "Gothic Organizer";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, iced::Task<Message>) {
        let mut app = Self {
            session: session_service::SessionService::new(),
            ..Default::default()
        };

        app.state.profile_choices = combo_box::State::new(app.session.profile_names.clone().unwrap_or_default());
        app.state.instance_choices = combo_box::State::new(app.session.instance_names.clone().unwrap_or_default());
        app.state.themes = session_service::SessionService::load_default_themes();
        app.state.theme_choices = combo_box::State::new(
            app.state
                .themes
                .iter()
                .map(|(_, t)| t.to_string())
                .collect(),
        );

        (app, iced::Task::done(Message::InitWindow))
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match &message {
            Message::InitWindow => {
                return self.session.init_window();
            }

            Message::ThemeSwitch(theme) => {
                self.session.theme_selected = Some(theme.clone());
            }

            Message::ProfileSelected(profile_name) => {
                let mut service = profile_service::ProfileService::new(&mut self.session, &mut self.state);
                return service.switch_profile(profile_name);
            }

            Message::InstanceSelected(instance) => {
                let mut service = profile_service::ProfileService::new(&mut self.session, &mut self.state);
                return service.switch_instance(instance);
            }

            Message::InstanceInput(input) => {
                self.state.instance_input = input.clone();
            }

            Message::InstanceAdd(profile_name) => {
                let mut service = profile_service::ProfileService::new(&mut self.session, &mut self.state);
                return service.add_instance_for_profile(profile_name);
            }

            Message::InstanceRemove(profile_name) => {
                let mut service = profile_service::ProfileService::new(&mut self.session, &mut self.state);
                service.remove_instance_from_profile(profile_name);
            }

            Message::FileToggle(path) => {
                let mut service = ui_service::UiService::new(&mut self.session, &mut self.state);
                service.toggle_state_recursive(Some(path));
            }

            Message::FileToggleAll => {
                let mut service = ui_service::UiService::new(&mut self.session, &mut self.state);
                service.toggle_state_recursive(None);
            }

            Message::SetGameDir(profile_name, path) => {
                let mut service = profile_service::ProfileService::new(&mut self.session, &mut self.state);
                return service.set_game_dir(profile_name.clone(), path.clone());
            }

            Message::ProfileDirInput(input) => {
                self.state.profile_directory_input = input.clone();
            }

            Message::TraverseIntoDir(path) => {
                self.fetch_ui_changes();
                self.state.current_directory = path.clone();
                let mut ui_service = ui_service::UiService::new(&mut self.session, &mut self.state);
                ui_service.reload_displayed_directory(Some(path.clone()))
            }

            Message::CurrentDirectoryUpdated => {
                let mut service = ui_service::UiService::new(&mut self.session, &mut self.state);
                service.reload_displayed_directory(None);
            }

            Message::LoadModsRequested => {
                let mut service = mod_service::ModService::new(&mut self.session);
                return service.reload_mods();
            }

            Message::ModToggle(name, new_state) => {
                self.fetch_ui_changes();
                let mut service = mod_service::ModService::new(&mut self.session);
                service.toggle_mod(name.clone(), *new_state);
                return iced::Task::done(Message::CurrentDirectoryUpdated);
            }

            Message::ModUninstall(name) => {
                let mut service = mod_service::ModService::new(&mut self.session);
                return service.remove_mod(name.clone());
            }

            Message::ModAdd(path) => {
                let mut service = mod_service::ModService::new(&mut self.session);
                return service.add_mod(path.clone());
            }

            Message::ModsDirInput(input) => {
                self.state.mods_directory_input = input.clone();
            }

            Message::SetModsDir(profile_name, path) => {
                let mut service = profile_service::ProfileService::new(&mut self.session, &mut self.state);
                return service.set_mods_dir(profile_name.clone(), path.clone());
            }

            Message::InvokeOptionsMenu => {
                return self.session.invoke_options_window();
            }

            Message::ErrorReturned(err) => {
                return self.session.exit_with_error(err.clone());
            }

            Message::Exit(wnd_id) => {
                self.fetch_ui_changes();
                return self.session.exit(wnd_id);
            }
        }

        iced::Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|event, _, id| match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => Some(Message::Exit(id)),
            iced::Event::Window(iced::window::Event::FileDropped(path)) => Some(Message::ModAdd(Some(path))),
            _ => None,
        })
    }

    pub fn theme(&self) -> iced::Theme {
        match &self.session.theme_selected {
            Some(theme) => self.state.themes.get(theme).cloned().unwrap_or_else(|| {
                log::warn!("Theme {theme} not found, defaulting to dark");
                iced::Theme::Dark
            }),
            None => iced::Theme::Dark,
        }
    }

    pub fn view(&self, id: iced::window::Id) -> iced::Element<Message> {
        if let Some((_, wnd_state)) = self
            .session
            .windows
            .iter()
            .find(|(wnd_id, _)| **wnd_id == Some(id))
        {
            if wnd_state.name == "options" {
                crate::gui::options_view::options_view(self)
            } else {
                crate::gui::editor_view::editor_view(self)
            }
        } else {
            iced::widget::container(iced::widget::text("no window")).into()
        }
    }

    fn fetch_ui_changes(&mut self) {
        let mut profile_service = profile_service::ProfileService::new(&mut self.session, &mut self.state);

        if let Err(err) = profile_service.update_instance_from_cache() {
            log::error!("Error updating instance cache: {err}");
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Exit(iced::window::Id),
    SetGameDir(Option<String>, Option<PathBuf>),
    SetModsDir(Option<String>, Option<PathBuf>),
    ProfileSelected(String),
    InstanceSelected(String),
    InstanceAdd(String),
    InstanceRemove(String),
    InstanceInput(String),
    ProfileDirInput(String),
    ModsDirInput(String),
    FileToggle(PathBuf),
    TraverseIntoDir(PathBuf),
    ThemeSwitch(String),
    ModToggle(String, bool),
    ModUninstall(String),
    ModAdd(Option<PathBuf>),
    ErrorReturned(error::SharedError),
    InitWindow,
    InvokeOptionsMenu,
    FileToggleAll,
    CurrentDirectoryUpdated,
    LoadModsRequested,
}
