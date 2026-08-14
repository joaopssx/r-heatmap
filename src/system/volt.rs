use crate::system::hwmon::{self, Reading};

pub struct VoltMonitor;

impl VoltMonitor {
    pub fn scan() -> Vec<Reading> {
        let volts = hwmon::scan("in", 0.001);
        log::info!("Found {} voltage sensors", volts.len());
        volts
    }
}
