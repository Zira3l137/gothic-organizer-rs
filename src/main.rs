mod app;
mod core;
mod error;
mod gui;
mod logger;
mod macros;

use std::path::PathBuf;

use clap::Parser;
use iced::daemon;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(short, long, default_value = None)]
    verbosity: Option<LevelFilter>,
    #[clap(short, long, default_value = None)]
    log_file: Option<PathBuf>,
    #[clap(short, long, default_value = None)]
    user_data_dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    logger::setup_logger(args.verbosity.unwrap_or(LevelFilter::ERROR), args.log_file.as_deref())?;

    daemon(core::constants::APP_TITLE, app::GothicOrganizer::update, app::GothicOrganizer::view)
        .theme(|state, _| app::GothicOrganizer::theme(state))
        .subscription(app::GothicOrganizer::subscription)
        .run_with(move || app::GothicOrganizer::new(args.user_data_dir))?;
    Ok(())
}
