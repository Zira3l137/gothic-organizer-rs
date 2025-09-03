use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::profile;
use crate::core::profile::Lookup;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationSession {
    pub active_profile: Option<String>,
    pub active_instance: Option<String>,
    pub active_renderer_backend: Option<RendererBackend>,
    pub active_zspy_config: Option<ZspyConfig>,
    pub mod_storage_dir: Option<PathBuf>,
    pub theme_selected: Option<String>,
    pub files: Lookup<PathBuf, profile::FileMetadata>,
    pub launch_options: Option<GameLaunchConfiguration>,
    pub error_notifications_enabled: bool,
}

impl std::default::Default for ApplicationSession {
    fn default() -> Self {
        Self {
            active_profile: None,
            active_instance: None,
            active_renderer_backend: None,
            active_zspy_config: None,
            mod_storage_dir: None,
            theme_selected: None,
            files: Default::default(),
            launch_options: None,
            error_notifications_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GameLaunchConfiguration {
    pub game_settings: GameSettings,
    pub parser_settings: ParserSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GameSettings {
    pub renderer: RendererBackend,
    pub zspy: ZspyConfig,
    pub is_marvin_mode_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ParserSettings {
    pub commands: Lookup<ParserCommand, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ZspyConfig {
    pub is_enabled: bool,
    pub verbosity: ZSpyVerbosity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ZSpyVerbosity {
    #[default]
    Off,
    Low,
    Medium,
    High,
    All,
}

impl From<ZSpyVerbosity> for u8 {
    fn from(value: ZSpyVerbosity) -> Self {
        match value {
            ZSpyVerbosity::Off => 0,
            ZSpyVerbosity::Low => 1,
            ZSpyVerbosity::Medium => 5,
            ZSpyVerbosity::High => 8,
            ZSpyVerbosity::All => 10,
        }
    }
}

impl From<u8> for ZSpyVerbosity {
    fn from(value: u8) -> Self {
        match value {
            0 => ZSpyVerbosity::Off,
            1..=4 => ZSpyVerbosity::Low,
            5..=7 => ZSpyVerbosity::Medium,
            8..=9 => ZSpyVerbosity::High,
            10 => ZSpyVerbosity::All,
            _ => ZSpyVerbosity::All,
        }
    }
}

impl std::fmt::Display for ZSpyVerbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZSpyVerbosity::Off => write!(f, "Off"),
            ZSpyVerbosity::Low => write!(f, "Low"),
            ZSpyVerbosity::Medium => write!(f, "Medium"),
            ZSpyVerbosity::High => write!(f, "High"),
            ZSpyVerbosity::All => write!(f, "All"),
        }
    }
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

    #[allow(dead_code)]
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
