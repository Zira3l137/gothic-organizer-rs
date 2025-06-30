mod app;
mod core;
mod error;
mod gui;
mod macros;

use clap::Parser;
use iced::daemon;
use log::LevelFilter;

use crate::app::GothicOrganizer;

#[derive(Debug, clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(short, long, default_value = None)]
    verbosity: Option<LevelFilter>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    setup_logger(args.verbosity.unwrap_or(LevelFilter::Error))?;

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

fn setup_logger(verbosity: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .filter_module(module_path!(), verbosity)
        .format_file(true)
        .format_line_number(true)
        .format_target(false)
        .format_timestamp(None)
        .init();

    log::debug!("Logger initialized with level: {verbosity}");
    Ok(())
}
