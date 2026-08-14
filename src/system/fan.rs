use crate::system::hwmon;
use crate::system::sysfs::Reading;

pub struct FanMonitor;

impl FanMonitor {
    pub fn scan() -> Vec<Reading> {
        let fans = hwmon::scan("fan", 1.0);
        log::info!("Found {} fan sensors", fans.len());
        fans
    }
}
