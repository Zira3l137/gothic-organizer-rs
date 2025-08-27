use std::path::PathBuf;

use crate::config;
use crate::error;
use crate::gui::options;

#[derive(Debug, Clone)]
pub enum Message {
    Profile(ProfileMessage),
    Mod(ModMessage),
    UI(UiMessage),
    Settings(SettingsMessage),
    Window(WindowMessage),
    System(SystemMessage),
}

#[derive(Debug, Clone)]
pub enum ProfileMessage {
    SetActive(String),
    SetActiveInstance(String),
    SetGameDir(Option<PathBuf>),
    AddInstance(String),
    RemoveActiveInstance,
    UpdateInstanceNameField(String),
    UpdateProfileDirField(String),
}

#[derive(Debug, Clone)]
pub enum ModMessage {
    Add(Option<PathBuf>),
    Toggle(String, bool),
    Uninstall(String),
    Reload,
    SetModsDir(Option<PathBuf>),
    UpdateModsDirField(String),
}

#[derive(Debug, Clone)]
pub enum UiMessage {
    UpdateActiveDir(PathBuf),
    ToggleFileEntry(PathBuf),
    ToggleAllFileEntries,
    ReloadDirEntries,
    SetTheme(String),
    SetOptionsMenu(options::menu::OptionsMenu),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    SetRendererBackend(config::RendererBackend),
    UpdateZspyLevel(u8),
    ToggleMarvinMode(bool),
    ToggleParserSetting(config::ParserCommand, bool),
    ToggleZSpyState(bool),
}

#[derive(Debug, Clone)]
pub enum WindowMessage {
    Close(iced::window::Id),
    Open(String),
    Initialize,
}

#[derive(Debug, Clone)]
pub enum SystemMessage {
    OpenRepository,
    ExitApplication,
    PanicWithError(error::SharedError),
    Idle,
}

impl From<ProfileMessage> for Message {
    fn from(msg: ProfileMessage) -> Self {
        Message::Profile(msg)
    }
}

impl From<ModMessage> for Message {
    fn from(msg: ModMessage) -> Self {
        Message::Mod(msg)
    }
}

impl From<UiMessage> for Message {
    fn from(msg: UiMessage) -> Self {
        Message::UI(msg)
    }
}

impl From<SettingsMessage> for Message {
    fn from(msg: SettingsMessage) -> Self {
        Message::Settings(msg)
    }
}

impl From<WindowMessage> for Message {
    fn from(msg: WindowMessage) -> Self {
        Message::Window(msg)
    }
}

impl From<SystemMessage> for Message {
    fn from(msg: SystemMessage) -> Self {
        Message::System(msg)
    }
}
