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
    pub fn new(label: String, input: PathBuf, scale: f32) -> Option<Self> {
        let value = read_number(&input)? * scale;

        Some(Self {
            label,
            value,
            min: None,
            max: None,
            input,
            scale,
        })
    }

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

pub fn read_text(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

pub fn read_number(path: &Path) -> Option<f32> {
    read_text(path)?.parse().ok()
}
