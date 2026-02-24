use fern::colors::{Color, ColoredLevelConfig};
use std::io;

pub fn setup_logger(level: &str) -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let log_level = match level.to_lowercase().as_str() {
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        _ => log::LevelFilter::Info,
    };

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log_level)
        .chain(io::stdout())
        .chain(fern::log_file("r-heatmap.log")?)
        .apply()?;

    Ok(())
}
