#![allow(dead_code)]

use std::path::PathBuf;

use iced::widget::combo_box;
use iced::window::Id;
use serde::{Deserialize, Serialize};

use crate::app::session;
use crate::core::constants;
use crate::core::profile;
use crate::core::profile::Lookup;
use crate::error;
use crate::gui::options;
use crate::load_profile;

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
    pub themes: Lookup<String, iced::Theme>,
    pub active_options_menu: options::menu::OptionsMenu,
    pub windows: Lookup<Option<Id>, WindowInfo>,
}

#[derive(Debug)]
pub struct ProfileState {
    pub profiles: Lookup<String, profile::Profile>,
    pub instance_name_field: String,
    pub profile_dir_field: String,
    pub profile_choices: combo_box::State<String>,
    pub instance_choices: combo_box::State<String>,
}

impl std::default::Default for ProfileState {
    fn default() -> Self {
        Self {
            profiles: constants::Profile::into_iter()
                .map(|profile_name| {
                    let name_str = (*profile_name).to_string();
                    let profile = load_profile!(&name_str)
                        .unwrap_or_else(|| profile::Profile::default().with_name(&name_str));
                    (name_str, profile)
                })
                .collect(),

            profile_choices: iced::widget::combo_box::State::new(
                constants::Profile::into_iter().map(|p| p.to_string()).collect(),
            ),
            instance_choices: Default::default(),
            instance_name_field: String::new(),
            profile_dir_field: String::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ModState {
    pub mods_dir_field: String,
}

#[derive(Debug, Default)]
pub struct SettingsState {
    pub zspy_level_field: u8,
    pub theme_choices: combo_box::State<String>,
    pub renderer_choices: combo_box::State<session::RendererBackend>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowInfo {
    pub name: String,
    pub is_closed: bool,
}

#[derive(Debug)]
pub struct ErrorState {
    pub active_errors: Lookup<uuid::Uuid, error::ErrorContext>,
    pub error_history: Vec<error::ErrorContext>,
    pub notifications_enabled: bool,
    pub max_history_size: usize,
}

impl std::default::Default for ErrorState {
    fn default() -> Self {
        Self {
            active_errors: Lookup::default(),
            error_history: Vec::new(),
            notifications_enabled: true,
            max_history_size: 100,
        }
    }
}

impl ErrorState {
    pub fn add_error(&mut self, error: error::ErrorContext) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();

        self.error_history.push(error.clone());
        if self.error_history.len() > self.max_history_size {
            self.error_history.remove(0);
        }

        self.active_errors.insert(id, error);

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
}
