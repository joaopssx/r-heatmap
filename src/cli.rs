use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Log level (info, debug, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Disable GPU monitoring
    #[arg(long)]
    pub no_gpu: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
