#[cfg(unix)]
use crate::system::meminfo;
use crate::system::reading::{self, Reading};
use sysinfo::System;

pub struct MemoryMonitor;

impl MemoryMonitor {
    pub fn refresh(sys: &mut System) {
        sys.refresh_memory();
    }

    pub fn get_usage(sys: &System) -> (u64, u64) {
        (sys.used_memory(), sys.total_memory())
    }

    pub fn scan() -> Vec<Reading> {
        let pools = scan_platform();
        reading::found("memory pools", &pools);
        pools
    }

    pub fn refresh_pools(pools: &mut [Reading]) {
        reading::update(pools, &scan_platform());
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    meminfo::pools()
}

#[cfg(not(unix))]
fn scan_platform() -> Vec<Reading> {
    Vec::new()
}
