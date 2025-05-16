mod constants;
mod error;
mod profile;
mod startup_window;

use iced::application;
use iced::window::icon;
use iced::window::Position;
use iced::window::Settings;
use iced::Size;

use crate::startup_window::StartupWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    println!("Hello, Auronen!");

    application(
        StartupWindow::WINDOW_TITLE,
        StartupWindow::update,
        StartupWindow::view,
    )
    .window(Settings {
        icon: icon::from_file("./resources/icon.ico").ok(),
        position: Position::Centered,
        resizable: false,
        ..Default::default()
    })
    .window_size(Size::from(StartupWindow::WINDOW_SIZE))
    .run_with(StartupWindow::new)?;
    Ok(())
}
