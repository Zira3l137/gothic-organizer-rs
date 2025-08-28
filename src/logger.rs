use std::fs::OpenOptions;
use std::path::Path;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_logger(
    verbosity: LevelFilter,
    file_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let log_file =
        OpenOptions::new().create(true).append(true).open(file_path.unwrap_or(Path::new("log.log")))?;

    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_ansi(true)
        .without_time()
        .with_filter(
            EnvFilter::new("")
                .add_directive(format!("{}={verbosity}", &package_name).parse()?)
                .add_directive("error".parse()?),
        );

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(log_file)
        .with_target(true)
        .with_file(true)
        .with_level(true)
        .with_line_number(true)
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::time())
        .with_filter(
            EnvFilter::new("")
                .add_directive(format!("{package_name}={verbosity}").parse()?)
                .add_directive("error".parse()?),
        );

    tracing_subscriber::registry().with(console_layer).with(file_layer).init();

    tracing::debug!("Logger initialized with level: {verbosity}");
    Ok(())
}
