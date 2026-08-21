#[cfg(unix)]
use crate::system::sysfs;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
const SLOW_READ: Duration = Duration::from_millis(1);
#[cfg(unix)]
const SLOW_INTERVAL: Duration = Duration::from_secs(2);

pub struct Reading {
    pub label: String,
    pub value: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub note: Option<String>,
    source: Source,
}

enum Source {
    Owner,
    #[cfg(unix)]
    File {
        file: File,
        scale: f32,
        cost: Duration,
        read: Instant,
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
        let mut file = File::open(input).ok()?;

        let read = Instant::now();
        let value = sysfs::reread(&mut file)? * scale;
        let cost = read.elapsed();

        Some(Self {
            label,
            value,
            min: None,
            max: None,
            note: None,
            source: Source::File {
                file,
                scale,
                cost,
                read,
            },
        })
    }

    pub fn refresh(&mut self) {
        let fresh = match &mut self.source {
            Source::Owner => None,
            #[cfg(unix)]
            Source::File {
                file,
                scale,
                cost,
                read,
            } => {
                if *cost > SLOW_READ && read.elapsed() < SLOW_INTERVAL {
                    return;
                }

                let started = Instant::now();
                let value = sysfs::reread(file).map(|value| value * *scale);

                *cost = started.elapsed();
                *read = started;

                value
            }
        };

        if let Some(value) = fresh {
            self.value = value;
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;

    fn slow_reading(path: &std::path::Path) -> Reading {
        let mut reading = Reading::from_file("slow".to_string(), path.to_path_buf(), 1.0).unwrap();

        if let Source::File { cost, .. } = &mut reading.source {
            *cost = SLOW_READ * 5;
        }

        reading
    }

    #[test]
    fn an_expensive_sensor_is_left_alone_between_intervals() {
        let path = std::env::temp_dir().join("r-heatmap-slow-sensor");
        fs::write(&path, "40\n").unwrap();

        let mut reading = slow_reading(&path);
        fs::write(&path, "80\n").unwrap();
        reading.refresh();

        assert_eq!(reading.value, 40.0);

        if let Source::File { read, .. } = &mut reading.source {
            *read = Instant::now() - SLOW_INTERVAL;
        }
        reading.refresh();
        fs::remove_file(&path).unwrap();

        assert_eq!(reading.value, 80.0);
    }

    #[test]
    fn a_cheap_sensor_is_read_every_time() {
        let path = std::env::temp_dir().join("r-heatmap-cheap-sensor");
        fs::write(&path, "40\n").unwrap();

        let mut reading = Reading::from_file("cheap".to_string(), path.clone(), 1.0).unwrap();
        fs::write(&path, "80\n").unwrap();
        reading.refresh();
        fs::remove_file(&path).unwrap();

        assert_eq!(reading.value, 80.0);
    }
}
