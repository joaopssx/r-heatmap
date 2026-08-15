#[cfg(unix)]
use crate::system::hwmon;
use crate::system::reading::{self, Reading};

pub struct VoltMonitor;

impl VoltMonitor {
    pub fn scan() -> Vec<Reading> {
        let volts = scan_platform();
        reading::found("voltage sensors", &volts);
        volts
    }

    pub fn refresh(volts: &mut [Reading]) {
        reading::refresh(volts);
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    hwmon::scan("in", 0.001)
}

#[cfg(not(unix))]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}
