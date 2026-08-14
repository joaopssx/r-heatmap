use std::fs;
use std::path::{Path, PathBuf};

pub struct Reading {
    pub label: String,
    pub value: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    input: PathBuf,
    scale: f32,
}

impl Reading {
    pub fn refresh(&mut self) {
        if let Some(value) = read_number(&self.input) {
            self.value = value * self.scale;
        }
    }
}

pub fn refresh(readings: &mut [Reading]) {
    for reading in readings {
        reading.refresh();
    }
}

pub fn scan(prefix: &str, scale: f32) -> Vec<Reading> {
    let mut readings = Vec::new();

    let chips = match fs::read_dir("/sys/class/hwmon") {
        Ok(chips) => chips,
        Err(e) => {
            log::warn!("Could not read /sys/class/hwmon: {}", e);
            return readings;
        }
    };

    for chip in chips.flatten() {
        let dir = chip.path();
        let name = read_text(&dir.join("name")).unwrap_or_else(|| "hwmon".to_string());

        let mut inputs: Vec<PathBuf> = match fs::read_dir(&dir) {
            Ok(entries) => entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| is_input(path, prefix))
                .collect(),
            Err(_) => continue,
        };
        inputs.sort();

        for input in inputs {
            let Some(value) = read_number(&input) else {
                continue;
            };

            let channel = match input.file_name().and_then(|f| f.to_str()) {
                Some(file) => file.trim_end_matches("_input").to_string(),
                None => continue,
            };

            let label = match read_text(&dir.join(format!("{channel}_label"))) {
                Some(label) if !label.is_empty() => format!("{name} {label}"),
                _ => format!("{name} {channel}"),
            };

            readings.push(Reading {
                label,
                value: value * scale,
                min: read_number(&dir.join(format!("{channel}_min"))).map(|v| v * scale),
                max: read_number(&dir.join(format!("{channel}_max"))).map(|v| v * scale),
                input,
                scale,
            });
        }
    }

    readings
}

fn is_input(path: &Path, prefix: &str) -> bool {
    match path.file_name().and_then(|f| f.to_str()) {
        Some(file) => file.starts_with(prefix) && file.ends_with("_input"),
        None => false,
    }
}

fn read_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn read_number(path: &Path) -> Option<f32> {
    read_text(path)?.parse().ok()
}
