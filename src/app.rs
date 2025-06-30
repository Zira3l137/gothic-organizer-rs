use std::path::PathBuf;

use iced::widget::combo_box::State;
use iced::Element;
use iced::Task;

use log::info;
use log::warn;
use scopeguard::defer;

use crate::core::logic;
use crate::core::profile::FileInfo;
use crate::core::profile::Lookup;
use crate::core::profile::Profile;

#[derive(Debug, Default)]
pub struct GothicOrganizer {
    pub profile_selected: Option<String>,
    pub instance_selected: Option<String>,
    pub profiles: Lookup<String, Profile>,
    pub files: Lookup<PathBuf, FileInfo>,
    pub theme: Option<String>,
    pub state: InnerState,
    pub mods_storage_dir: Option<PathBuf>,
    pub windows: Lookup<Option<iced::window::Id>, WindowState>,
}

#[derive(Debug, Default)]
pub struct InnerState {
    pub instance_input: Option<String>,
    pub profile_directory_input: String,
    pub profile_choices: State<String>,
    pub themes: Lookup<String, iced::Theme>,
    pub theme_choices: State<String>,
    pub instance_choices: State<String>,
    pub current_directory_entries: Vec<(PathBuf, FileInfo)>,
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
        info!("Initializing app");
        let mut app = Self::default();

        info!("Loading last session");
        if let Err(err) = logic::try_reload_last_session(&mut app) {
            warn!("Failed to load last session: {err}");
        }

        app.state.themes = logic::load_default_themes();
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
        defer! {
            info!("Message: {message:?}");
        }

        match &message {
            Message::InitWindow => {
                return logic::init_window(self);
            }

            Message::ThemeSwitch(theme) => {
                self.theme = Some(theme.clone());
            }

            Message::ProfileSelected(profile_name) => {
                return logic::switch_profile(self, profile_name);
            }

            Message::InstanceSelected(instance) => {
                return logic::select_instance(self, instance);
            }

            Message::InstanceInput(input) => {
                self.state.instance_input = Some(input.clone());
            }

            Message::InstanceAdd(profile_name) => {
                return logic::add_instance_for_profile(self, profile_name);
            }

            Message::InstanceRemove(profile_name) => {
                logic::remove_instance_from_profile(self, profile_name);
            }

            Message::FileToggle(path) => {
                logic::toggle_state_recursive(self, Some(path));
            }

            Message::FileToggleAll => {
                logic::toggle_state_recursive(self, None);
            }

            Message::BrowseGameDir(profile_name, path) => {
                return logic::set_game_dir(self, profile_name.clone(), path.clone());
            }

            Message::ProfileDirInput(input) => {
                self.state.profile_directory_input = input.clone();
            }

            Message::TraverseIntoDir(path) => {
                logic::write_current_changes(self);
                self.state.current_directory = path.clone();
                logic::load_files(self, Some(path.clone()))
            }

            Message::RefreshFiles => {
                logic::load_files(self, None);
            }

            Message::ModToggle(_) => {
                return Task::none();
            }

            Message::ModUninstall(name) => {
                logic::remove_mod(self, None, None, name.clone());
            }

            Message::ModAdd(path) => {
                return logic::add_mod(self, None, None, path.clone());
            }

            Message::InvokeOptionsMenu => {
                return logic::invoke_options_window(self);
            }

            Message::Exit(wnd_id) => {
                return logic::exit(self, wnd_id);
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
            Some(theme) => self
                .state
                .themes
                .get(theme)
                .cloned()
                .unwrap_or(iced::Theme::Dark),
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
    BrowseGameDir(Option<String>, Option<PathBuf>),
    ProfileSelected(String),
    InstanceSelected(String),
    InstanceAdd(String),
    InstanceRemove(String),
    InstanceInput(String),
    ProfileDirInput(String),
    FileToggle(PathBuf),
    TraverseIntoDir(PathBuf),
    ThemeSwitch(String),
    ModToggle(String),
    ModUninstall(String),
    ModAdd(Option<PathBuf>),
    InitWindow,
    InvokeOptionsMenu,
    FileToggleAll,
    RefreshFiles,
}
