mod app;
mod core;
mod error;
mod gui;
mod macros;

use iced::daemon;

use app::GothicOrganizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    daemon(
        GothicOrganizer::WINDOW_TITLE,
        GothicOrganizer::update,
        GothicOrganizer::view,
    )
    .theme(|state, _| GothicOrganizer::theme(state))
    .subscription(GothicOrganizer::subscription)
    .run_with(GothicOrganizer::new)?;
    Ok(())
}
