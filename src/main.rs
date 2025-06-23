use iced::application;
use iced::window::icon;
use iced::window::Position;
use iced::window::Settings;
use iced::Size;

use crate::editor::Editor;

mod constants;
mod cutstom_widgets;
mod editor;
mod error;
mod helpers;
mod macros;
mod profile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    application(Editor::WINDOW_TITLE, Editor::update, Editor::view)
        .window(Settings {
            icon: icon::from_file("./resources/icon.ico").ok(),
            position: Position::Centered,
            resizable: false,
            exit_on_close_request: false,
            ..Default::default()
        })
        .subscription(Editor::subscription)
        .window_size(Size::from(Editor::WINDOW_SIZE))
        .run_with(Editor::new)?;
    Ok(())
}
