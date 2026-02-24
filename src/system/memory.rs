use sysinfo::System;

pub struct MemoryMonitor;

impl MemoryMonitor {
    pub fn refresh(sys: &mut System) {
        sys.refresh_memory();
    }

    pub fn get_usage(sys: &System) -> (u64, u64) {
        (sys.used_memory(), sys.total_memory())
    }
}
