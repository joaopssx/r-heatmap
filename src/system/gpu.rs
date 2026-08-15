#[cfg(unix)]
use crate::system::drm;
use crate::system::reading::{self, Reading};
#[cfg(windows)]
use crate::system::windows::perf;

pub struct GpuMonitor;

impl GpuMonitor {
    pub fn scan() -> Vec<Reading> {
        let gpus = scan_platform();
        reading::found("GPUs reporting usage", &gpus);
        gpus
    }

    pub fn refresh(gpus: &mut [Reading]) {
        refresh_platform(gpus);
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    drm::scan()
}

#[cfg(unix)]
fn refresh_platform(gpus: &mut [Reading]) {
    reading::refresh(gpus);
}

#[cfg(windows)]
fn scan_platform() -> Vec<Reading> {
    perf::gpu_usage()
}

#[cfg(windows)]
fn refresh_platform(gpus: &mut [Reading]) {
    reading::update(gpus, &perf::gpu_usage());
}
