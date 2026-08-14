use crate::system::hwmon::{self, Reading};

pub struct FanMonitor;

impl FanMonitor {
    pub fn scan() -> Vec<Reading> {
        let fans = hwmon::scan("fan");
        log::info!("Found {} fan sensors", fans.len());
        fans
    }

    pub fn refresh(fans: &mut [Reading]) {
        for fan in fans {
            fan.refresh();
        }
    }
}
