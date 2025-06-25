mod core;
mod gui;
mod macros;

use iced::application;
use iced::window::icon;
use iced::window::Position;
use iced::window::Settings;
use iced::Size;

use gui::app::GothicOrganizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    application(
        GothicOrganizer::WINDOW_TITLE,
        GothicOrganizer::update,
        GothicOrganizer::view,
    )
    .window(Settings {
        icon: icon::from_file("./resources/icon.ico").ok(),
        position: Position::Centered,
        resizable: false,
        exit_on_close_request: false,
        ..Default::default()
    })
    .subscription(GothicOrganizer::subscription)
    .theme(|_| iced::theme::Theme::CatppuccinMocha)
    .window_size(Size::from(GothicOrganizer::WINDOW_SIZE))
    .run_with(GothicOrganizer::new)?;
    Ok(())
}
