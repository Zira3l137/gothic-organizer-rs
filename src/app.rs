use std::path::PathBuf;

use iced::widget::combo_box::State;
use iced::Element;
use iced::Task;

use crate::core::logic::app_lifecycle;
use crate::core::logic::mod_management;
use crate::core::logic::profile_management;
use crate::core::logic::ui_logic;
use crate::core::lookup::Lookup;
use crate::core::profile;
use crate::error;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub profile_selected: Option<String>,
    pub instance_selected: Option<String>,
    pub profiles: Lookup<String, profile::Profile>,
    pub files: Lookup<PathBuf, profile::FileInfo>,
    pub theme: Option<String>,
    pub state: InnerState,
    pub mod_storage_dir: Option<PathBuf>,
    pub windows: Lookup<Option<iced::window::Id>, WindowState>,
}

#[derive(Debug, Default)]
pub struct InnerState {
    pub instance_input: Option<String>,
    pub profile_directory_input: String,
    pub mods_directory_input: String,
    pub profile_choices: State<String>,
    pub themes: Lookup<String, iced::Theme>,
    pub theme_choices: State<String>,
    pub instance_choices: State<String>,
    pub current_directory_entries: Vec<(PathBuf, profile::FileInfo)>,
    pub current_directory: PathBuf,
}

#[derive(Debug, Default)]
pub struct WindowState {
    pub name: String,
    pub closed: bool,
}

impl GothicOrganizer {
    pub const WINDOW_TITLE: &str = "Gothic Organizer";
    pub const WINDOW_SIZE: (f32, f32) = (768.0, 768.0);

    pub fn new() -> (Self, Task<Message>) {
        let mut app = Self::default();

        if let Err(err) = app_lifecycle::try_reload_last_session(&mut app) {
            log::warn!("Failed to load last session: {err}");
        }

        app.state.themes = app_lifecycle::load_default_themes();
        app.state.theme_choices = State::new(
            app.state
                .themes
                .iter()
                .map(|(_, t)| t.to_string())
                .collect(),
        );

        (app, Task::done(Message::InitWindow))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match &message {
            Message::InitWindow => {
                return app_lifecycle::init_window(self);
            }

            Message::ThemeSwitch(theme) => {
                log::debug!("Switching theme to {theme}");
                self.theme = Some(theme.clone());
            }

            Message::ProfileSelected(profile_name) => {
                return profile_management::switch_profile(self, profile_name);
            }

            Message::InstanceSelected(instance) => {
                return profile_management::select_instance(self, instance);
            }

            Message::InstanceInput(input) => {
                self.state.instance_input = Some(input.clone());
            }

            Message::InstanceAdd(profile_name) => {
                return profile_management::add_instance_for_profile(self, profile_name);
            }

            Message::InstanceRemove(profile_name) => {
                profile_management::remove_instance_from_profile(self, profile_name);
            }

            Message::FileToggle(path) => {
                ui_logic::toggle_state_recursive(self, Some(path));
            }

            Message::FileToggleAll => {
                ui_logic::toggle_state_recursive(self, None);
            }

            Message::SetGameDir(profile_name, path) => {
                return profile_management::set_game_dir(self, profile_name.clone(), path.clone());
            }

            Message::ProfileDirInput(input) => {
                self.state.profile_directory_input = input.clone();
            }

            Message::TraverseIntoDir(path) => {
                profile_management::write_changes_to_instance(self);
                self.state.current_directory = path.clone();
                ui_logic::load_files(self, Some(path.clone()))
            }

            Message::RefreshFiles => {
                ui_logic::load_files(self, None);
            }

            Message::ModToggle(_) => {
                return Task::none();
            }

            Message::ModUninstall(name) => {
                return mod_management::remove_mod(self, name.clone());
            }

            Message::ModAdd(path) => {
                return mod_management::add_mod(self, path.clone());
            }

            Message::ModsDirInput(input) => {
                self.state.mods_directory_input = input.clone();
            }

            Message::SetModsDir(profile_name, path) => {
                return profile_management::set_mods_dir(self, profile_name.clone(), path.clone());
            }

            Message::LoadMods => {
                return mod_management::load_mods(self);
            }

            Message::InvokeOptionsMenu => {
                return app_lifecycle::invoke_options_window(self);
            }

            Message::ReturnError(err) => {
                return app_lifecycle::exit_with_error(self, err.clone());
            }

            Message::Exit(wnd_id) => {
                return app_lifecycle::exit(self, wnd_id);
            }
        }

        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|event, _, id| match event {
            iced::Event::Window(iced::window::Event::CloseRequested) => Some(Message::Exit(id)),
            iced::Event::Window(iced::window::Event::FileDropped(path)) => Some(Message::ModAdd(Some(path))),
            _ => None,
        })
    }

    pub fn theme(&self) -> iced::Theme {
        match &self.theme {
            Some(theme) => self.state.themes.get(theme).cloned().unwrap_or_else(|| {
                log::warn!("Theme {theme} not found, defaulting to dark");
                iced::Theme::Dark
            }),
            None => iced::Theme::Dark,
        }
    }

    pub fn view(&self, id: iced::window::Id) -> Element<Message> {
        if let Some((_, wnd_state)) = self.windows.iter().find(|(wnd_id, _)| **wnd_id == Some(id)) {
            if wnd_state.name == "options" {
                crate::gui::options_view::options_view(self)
            } else {
                crate::gui::editor_view::editor_view(self)
            }
        } else {
            iced::widget::container(iced::widget::text("no window")).into()
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
    ModToggle(String),
    ModUninstall(String),
    ModAdd(Option<PathBuf>),
    ReturnError(error::SharedError),
    LoadMods,
    InitWindow,
    InvokeOptionsMenu,
    FileToggleAll,
    RefreshFiles,
}
