#[cfg(unix)]
use crate::system::hwmon;
use crate::system::reading::{self, Reading};
use sysinfo::Components;

pub struct TempMonitor;

impl TempMonitor {
    pub fn scan(components: &Components) -> Vec<Reading> {
        let temps = scan_platform(components);
        reading::found("temperature sensors", &temps);
        temps
    }

    pub fn refresh(temps: &mut [Reading], components: &mut Components) {
        refresh_platform(temps, components);
    }
}

#[cfg(unix)]
fn scan_platform(_components: &Components) -> Vec<Reading> {
    hwmon::scan("temp", 0.001)
}

#[cfg(unix)]
fn refresh_platform(temps: &mut [Reading], _components: &mut Components) {
    reading::refresh(temps);
}

#[cfg(not(unix))]
fn scan_platform(components: &Components) -> Vec<Reading> {
    components
        .iter()
        .map(|component| {
            Reading::new(
                component.label().to_string(),
                component.temperature().unwrap_or(0.0),
            )
        })
        .collect()
}

#[cfg(not(unix))]
fn refresh_platform(temps: &mut [Reading], components: &mut Components) {
    components.refresh(true);
    reading::update(temps, &scan_platform(components));
}
