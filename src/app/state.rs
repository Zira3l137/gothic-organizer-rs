#![allow(dead_code)]

use std::path::PathBuf;

use iced::widget::combo_box;

use crate::config;
use crate::core::lookup;
use crate::core::profile;
use crate::error;
use crate::gui::options;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub ui: UiState,
    pub profile: ProfileState,
    pub mod_management: ModState,
    pub settings: SettingsState,
    pub errors: ErrorState,
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

#[derive(Debug, Default)]
pub struct ErrorState {
    pub active_errors: lookup::Lookup<uuid::Uuid, error::ErrorContext>,
    pub error_history: Vec<error::ErrorContext>,
    pub notifications_enabled: bool,
    pub max_history_size: usize,
}

impl ErrorState {
    pub fn new() -> Self {
        Self {
            active_errors: lookup::Lookup::new(),
            error_history: Vec::new(),
            notifications_enabled: true,
            max_history_size: 100,
        }
    }

    pub fn add_error(&mut self, error: error::ErrorContext) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();

        self.error_history.push(error.clone());
        if self.error_history.len() > self.max_history_size {
            self.error_history.remove(0);
        }

        if !self.should_auto_dismiss(&error) {
            self.active_errors.insert(id, error);
        }

        id
    }

    pub fn dismiss_error(&mut self, id: uuid::Uuid) -> Option<error::ErrorContext> {
        self.active_errors.remove(&id)
    }

    pub fn clear_all(&mut self) {
        self.active_errors.clear();
    }

    pub fn get_errors(&self) -> Vec<&error::ErrorContext> {
        self.active_errors.values().collect()
    }

    fn should_auto_dismiss(&self, error: &error::ErrorContext) -> bool {
        error.recoverable
    }
}
