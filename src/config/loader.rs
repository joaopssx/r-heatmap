use crate::config::Config;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load(path: Option<&str>) -> Config {
    match path {
        Some(p) => read_or_default(Path::new(p)),
        None => match default_path() {
            Some(p) => read_or_default(&p),
            None => Config::default(),
        },
    }
}

fn default_path() -> Option<PathBuf> {
    let local = PathBuf::from("config.toml");
    if local.exists() {
        return Some(local);
    }

    Some(config_home()?.join("r-heatmap").join("config.toml"))
}

#[cfg(windows)]
fn config_home() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(not(windows))]
fn config_home() -> Option<PathBuf> {
    match std::env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => Some(PathBuf::from(dir)),
        None => Some(PathBuf::from(std::env::var_os("HOME")?).join(".config")),
    }
}

fn read_or_default(path: &Path) -> Config {
    if !path.exists() {
        log::warn!("Config file {} not found. Using defaults.", path.display());
        return Config::default();
    }

    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => {
                log::info!("Loaded configuration from {}", path.display());
                config
            }
            Err(e) => {
                log::error!("Error parsing {}: {}. Using defaults.", path.display(), e);
                Config::default()
            }
        },
        Err(e) => {
            log::error!("Could not read {}: {}. Using defaults.", path.display(), e);
            Config::default()
        }
    }
}
