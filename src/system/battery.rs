#[cfg(unix)]
use crate::system::power_supply;
use crate::system::reading::{self, Reading};

pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn scan() -> Vec<Reading> {
        let batteries = scan_platform();
        reading::found("batteries", &batteries);
        batteries
    }

    pub fn refresh(batteries: &mut [Reading]) {
        reading::update(batteries, &scan_platform());
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    power_supply::charge()
}

#[cfg(not(unix))]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}
