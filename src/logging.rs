pub fn setup_logger(level: &str) -> Result<(), fern::InitError> {
    let log_level = match level.to_lowercase().as_str() {
        "debug" => log::LevelFilter::Debug,
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        _ => log::LevelFilter::Info,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(fern::log_file("r-heatmap.log")?)
        .apply()?;

    Ok(())
}
