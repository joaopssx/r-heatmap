#[cfg(unix)]
use crate::system::sysfs;
#[cfg(unix)]
use std::path::PathBuf;

#[derive(Clone)]
pub struct Reading {
    pub label: String,
    pub value: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub note: Option<String>,
    source: Source,
}

#[derive(Clone)]
enum Source {
    Owner,
    #[cfg(unix)]
    File {
        input: PathBuf,
        scale: f32,
    },
}

impl Reading {
    pub fn new(label: String, value: f32) -> Self {
        Self {
            label,
            value,
            min: None,
            max: None,
            note: None,
            source: Source::Owner,
        }
    }

    #[cfg(unix)]
    pub fn from_file(label: String, input: PathBuf, scale: f32) -> Option<Self> {
        let value = sysfs::read_number(&input)? * scale;

        Some(Self {
            label,
            value,
            min: None,
            max: None,
            note: None,
            source: Source::File { input, scale },
        })
    }

    pub fn refresh(&mut self) {
        match &self.source {
            Source::Owner => {}
            #[cfg(unix)]
            Source::File { input, scale } => {
                if let Some(value) = sysfs::read_number(input) {
                    self.value = value * scale;
                }
            }
        }
    }
}

pub fn found(kind: &str, readings: &[Reading]) {
    log::info!("Found {} {}", readings.len(), kind);

    for reading in readings {
        log::debug!("{}: {} = {}", kind, reading.label, reading.value);
    }
}

pub fn refresh(readings: &mut [Reading]) {
    for reading in readings {
        reading.refresh();
    }
}

pub fn update(readings: &mut [Reading], fresh: &[Reading]) {
    for reading in readings {
        if let Some(fresh) = fresh.iter().find(|f| f.label == reading.label) {
            reading.value = fresh.value;
            reading.note = fresh.note.clone();
        }
    }
}
