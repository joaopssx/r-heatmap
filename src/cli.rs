use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file (default: ./config.toml, then the user config directory)
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<String>,

    /// Log level (debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Disable GPU monitoring
    #[arg(long)]
    pub no_gpu: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
