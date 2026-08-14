use crate::system::sysfs::{self, Reading};
use std::fs;
use std::path::{Path, PathBuf};

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
        let name = sysfs::read_text(&dir.join("name")).unwrap_or_else(|| "hwmon".to_string());

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
            let channel = match input.file_name().and_then(|f| f.to_str()) {
                Some(file) => file.trim_end_matches("_input").to_string(),
                None => continue,
            };

            let label = match sysfs::read_text(&dir.join(format!("{channel}_label"))) {
                Some(label) if !label.is_empty() => format!("{name} {label}"),
                _ => format!("{name} {channel}"),
            };

            let Some(mut reading) = Reading::new(label, input, scale) else {
                continue;
            };

            reading.min =
                sysfs::read_number(&dir.join(format!("{channel}_min"))).map(|v| v * scale);
            reading.max =
                sysfs::read_number(&dir.join(format!("{channel}_max"))).map(|v| v * scale);
            readings.push(reading);
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
