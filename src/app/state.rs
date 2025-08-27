use std::path::PathBuf;

use iced::widget::combo_box;

use crate::config;
use crate::core::lookup;
use crate::core::profile;
use crate::gui::options;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub ui: UiState,
    pub profile: ProfileState,
    pub mod_management: ModState,
    pub settings: SettingsState,
}

#[derive(Debug, Default)]
pub struct UiState {
    pub current_dir: PathBuf,
    pub dir_entries: Vec<(PathBuf, profile::FileMetadata)>,
    pub themes: lookup::Lookup<String, iced::Theme>,
    pub active_options_menu: options::menu::OptionsMenu,
}

#[derive(Debug, Default)]
pub struct ProfileState {
    pub instance_name_field: String,
    pub profile_dir_field: String,
    pub profile_choices: combo_box::State<String>,
    pub instance_choices: combo_box::State<String>,
}

#[derive(Debug, Default)]
pub struct ModState {
    pub mods_dir_field: String,
}

#[derive(Debug, Default)]
pub struct SettingsState {
    pub zspy_level_field: u8,
    pub theme_choices: combo_box::State<String>,
    pub renderer_choices: combo_box::State<config::RendererBackend>,
}

#[derive(Debug, Default, Clone)]
pub struct WindowState {
    pub name: String,
    pub is_closed: bool,
}
