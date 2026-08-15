// Configuration management - by joaopssx
pub mod loader;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub style: StyleConfig,
    pub thresholds: ThresholdsConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneralConfig {
    pub refresh_rate_ms: u64,
    pub github_repo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StyleConfig {
    pub border_color: String,
    pub header_color: String,
    pub sensor_label_contains: Vec<String>,
    #[serde(default = "default_disk_labels")]
    pub disk_label_contains: Vec<String>,
}

fn default_disk_labels() -> Vec<String> {
    vec!["nvme".to_string(), "drivetemp".to_string()]
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThresholdsConfig {
    pub cold: f32,
    pub warm: f32,
    pub hot: f32,
    pub critical: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                refresh_rate_ms: 500,
                github_repo: "joaopssx/r-heatmap".to_string(),
            },
            style: StyleConfig {
                border_color: "Cyan".to_string(),
                header_color: "Yellow".to_string(),
                sensor_label_contains: vec![
                    "core".to_string(),
                    "tctl".to_string(),
                    "tccd".to_string(),
                    "cpu".to_string(),
                    "package".to_string(),
                    "die".to_string(),
                    "computer".to_string(),
                ],
                disk_label_contains: default_disk_labels(),
            },
            thresholds: ThresholdsConfig {
                cold: 40.0,
                warm: 60.0,
                hot: 80.0,
                critical: 90.0,
            },
        }
    }
}

pub fn load_config(path: Option<&str>) -> Config {
    loader::load(path)
}
