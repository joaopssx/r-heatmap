use crate::system::reading::Reading;
use crate::system::sysfs;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan(prefix: &str, scale: f32) -> Vec<Reading> {
    let mut readings = Vec::new();

    let Some(root) = sysfs::class_dir("hwmon") else {
        return readings;
    };

    let chips = match fs::read_dir(&root) {
        Ok(chips) => chips,
        Err(e) => {
            log::warn!("Could not read {}: {}", root.display(), e);
            return readings;
        }
    };

    for chip in chips.flatten() {
        let dir = chip.path();
        let name = sysfs::read_text(&dir.join("name")).unwrap_or_else(|| "hwmon".to_string());

        readings.extend(scan_chip(&dir, &name, prefix, scale));
    }

    readings
}

pub fn scan_chip(dir: &Path, name: &str, prefix: &str, scale: f32) -> Vec<Reading> {
    let mut readings = Vec::new();

    let mut inputs: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(entries) => entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| is_input(path, prefix))
            .collect(),
        Err(_) => return readings,
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

        let Some(mut reading) = Reading::from_file(label, input, scale) else {
            continue;
        };

        reading.min = sysfs::read_number(&dir.join(format!("{channel}_min"))).map(|v| v * scale);
        reading.max = sysfs::read_number(&dir.join(format!("{channel}_max"))).map(|v| v * scale);
        readings.push(reading);
    }

    readings
}

pub fn chip_dir(device: &Path) -> Option<PathBuf> {
    let mut chips: Vec<PathBuf> = fs::read_dir(device.join("hwmon"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    chips.sort();

    chips.into_iter().next()
}

fn is_input(path: &Path, prefix: &str) -> bool {
    match path.file_name().and_then(|f| f.to_str()) {
        Some(file) => file.starts_with(prefix) && file.ends_with("_input"),
        None => false,
    }
}
