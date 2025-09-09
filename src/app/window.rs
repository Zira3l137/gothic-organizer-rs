#![allow(dead_code)]

use derive_more::Display;
use serde::{Deserialize, Serialize};

use super::{GothicOrganizer, message::Message};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WindowInfo {
    pub window_type: ApplicationWindow,
    pub is_closed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize, Hash, Default)]
pub enum ApplicationWindow {
    #[default]
    #[display("editor")]
    Editor,

    #[display("options")]
    Options,

    #[display("conflicts")]
    Conflicts,

    #[display("logs")]
    Logs,
}

impl From<&str> for ApplicationWindow {
    fn from(value: &str) -> Self {
        match value {
            "editor" => ApplicationWindow::Editor,
            "options" => ApplicationWindow::Options,
            "conflicts" => ApplicationWindow::Conflicts,
            "logs" => ApplicationWindow::Logs,
            _ => unreachable!(),
        }
    }
}

impl ApplicationWindow {
    pub fn into_iter() -> std::slice::Iter<'static, ApplicationWindow> {
        static WINDOWS: [ApplicationWindow; 3] =
            [ApplicationWindow::Editor, ApplicationWindow::Options, ApplicationWindow::Conflicts];
        WINDOWS.iter()
    }

    pub fn name(&self) -> String {
        self.to_string()
    }

    pub fn default_size(&self) -> iced::Size {
        match self {
            ApplicationWindow::Editor => iced::Size { width: 768.0, height: 768.0 },
            ApplicationWindow::Options => iced::Size { width: 768.0, height: 460.0 },
            ApplicationWindow::Conflicts => iced::Size { width: 768.0, height: 460.0 },
            ApplicationWindow::Logs => iced::Size { width: 512.0, height: 512.0 },
        }
    }

    pub fn default_position(&self) -> iced::window::Position {
        match self {
            ApplicationWindow::Editor => iced::window::Position::Centered,
            ApplicationWindow::Options => iced::window::Position::Centered,
            ApplicationWindow::Conflicts => iced::window::Position::Centered,
            ApplicationWindow::Logs => iced::window::Position::Centered,
        }
    }

    pub fn view<'a>(&self, app: &'a GothicOrganizer) -> iced::Element<'a, Message> {
        match self {
            ApplicationWindow::Editor => crate::gui::editor::editor_view(app),
            ApplicationWindow::Options => crate::gui::options::options_view(app),
            ApplicationWindow::Conflicts => crate::gui::conflicts::conflicts_view(app),
            ApplicationWindow::Logs => crate::gui::logs::logs_view(app),
        }
    }
}
