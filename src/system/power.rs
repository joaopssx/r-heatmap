#[cfg(unix)]
use crate::system::rapl;
use crate::system::reading::{self, Reading};

pub struct PowerMonitor;

impl PowerMonitor {
    pub fn scan() -> Vec<Reading> {
        let power = scan_platform();
        reading::found("power zones", &power);
        power
    }

    pub fn refresh(power: &mut [Reading]) {
        reading::update(power, &scan_platform());
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    rapl::power()
}

#[cfg(not(unix))]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}
