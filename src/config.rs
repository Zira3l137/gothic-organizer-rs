use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::core::lookup::Lookup;
use crate::core::profile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub mod_storage_dir: PathBuf,
    pub theme: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Session {
    pub selected_profile: Option<String>,
    pub selected_instance: Option<String>,
    pub launch_options: Option<LaunchOptions>,
    pub cache: Option<Lookup<PathBuf, profile::FileInfo>>,
}

impl AsRef<Session> for Session {
    fn as_ref(&self) -> &Session {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LaunchOptions {
    pub game_settings: GameSettings,
    pub parser_settings: ParserSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GameSettings {
    pub renderer: RendererBackend,
    pub zspy: ZSpyMessagesLevel,
    pub marvin_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ParserSettings {
    pub commands: Lookup<ParserCommand, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ZSpyMessagesLevel {
    #[default]
    Off,
    Low,
    Medium,
    High,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RendererBackend {
    #[default]
    D3D8,
    D3D11,
}

impl std::fmt::Display for RendererBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererBackend::D3D8 => write!(f, "D3D8"),
            RendererBackend::D3D11 => write!(f, "D3D11"),
        }
    }
}

impl RendererBackend {
    pub fn into_iter() -> std::slice::Iter<'static, Self> {
        static VALUES: [RendererBackend; 2] = [RendererBackend::D3D8, RendererBackend::D3D11];
        VALUES.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum ParserCommand {
    #[default]
    Game,
    Ou,
    Sfx,
    Pfx,
    Vfx,
    Camera,
    Menu,
    Music,
}

impl std::fmt::Display for ParserCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserCommand::Game => write!(f, "Game"),
            ParserCommand::Ou => write!(f, "Ou"),
            ParserCommand::Sfx => write!(f, "Sfx"),
            ParserCommand::Pfx => write!(f, "Pfx"),
            ParserCommand::Vfx => write!(f, "Vfx"),
            ParserCommand::Camera => write!(f, "Camera"),
            ParserCommand::Menu => write!(f, "Menu"),
            ParserCommand::Music => write!(f, "Music"),
        }
    }
}

impl ParserCommand {
    pub fn into_iter() -> std::slice::Iter<'static, Self> {
        static COMMANDS: [ParserCommand; 8] = [
            ParserCommand::Game,
            ParserCommand::Ou,
            ParserCommand::Sfx,
            ParserCommand::Pfx,
            ParserCommand::Vfx,
            ParserCommand::Camera,
            ParserCommand::Menu,
            ParserCommand::Music,
        ];
        COMMANDS.iter()
    }

    pub fn into_argument(self) -> String {
        match self {
            ParserCommand::Game => "zReparse_Game".to_owned(),
            ParserCommand::Ou => "zReparse_Ou".to_owned(),
            ParserCommand::Sfx => "zReparse_SFX".to_owned(),
            ParserCommand::Pfx => "zReparse_PFX".to_owned(),
            ParserCommand::Vfx => "zReparse_VFX".to_owned(),
            ParserCommand::Camera => "zReparse_Camera".to_owned(),
            ParserCommand::Menu => "zReparse_Menu".to_owned(),
            ParserCommand::Music => "zReparse_Music".to_owned(),
        }
    }
}
