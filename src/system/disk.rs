use crate::system::reading::Reading;
#[cfg(windows)]
use crate::system::{reading, windows::storage};

pub struct DiskMonitor;

impl DiskMonitor {
    pub fn scan() -> Vec<Reading> {
        scan_platform()
    }

    pub fn refresh(disks: &mut [Reading]) {
        refresh_platform(disks);
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}

#[cfg(unix)]
fn refresh_platform(_disks: &mut [Reading]) {}

#[cfg(windows)]
fn scan_platform() -> Vec<Reading> {
    let disks = storage::temperatures();
    reading::found("disk temperature sensors", &disks);
    disks
}

#[cfg(windows)]
fn refresh_platform(disks: &mut [Reading]) {
    reading::update(disks, &storage::temperatures());
}
