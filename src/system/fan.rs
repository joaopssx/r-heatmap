#[cfg(unix)]
use crate::system::hwmon;
use crate::system::reading::{self, Reading};

pub struct FanMonitor;

impl FanMonitor {
    pub fn scan() -> Vec<Reading> {
        let fans = scan_platform();
        reading::found("fan sensors", &fans);
        fans
    }

    pub fn refresh(fans: &mut [Reading]) {
        reading::refresh(fans);
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    hwmon::scan("fan", 1.0)
}

#[cfg(not(unix))]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}
