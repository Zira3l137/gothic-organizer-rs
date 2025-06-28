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
    #[clap(short, long, default_value = "0")]
    verbosity: Option<u8>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    setup_logger(args.verbosity.unwrap_or(0))?;

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

fn setup_logger(verbosity: u8) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_module(module_path!(), log_level)
        .format_timestamp(None)
        .init();

    log::debug!("Logger initialized with level: {log_level}");
    Ok(())
}
