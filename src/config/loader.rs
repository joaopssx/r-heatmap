use crate::config::Config;
use std::fs;
use std::path::Path;

pub fn load_from_file(path_str: &str) -> Config {
    let config_path = Path::new(path_str);
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(config_path) {
            match toml::from_str::<Config>(&content) {
                Ok(config) => {
                    log::info!("Loaded configuration from {}", path_str);
                    return config;
                }
                Err(e) => {
                    log::error!("Error parsing {}: {}. Using defaults.", path_str, e);
                }
            }
        }
    } else {
        log::warn!("Config file {} not found. Creating default.", path_str);
    }
    Config::default()
}
