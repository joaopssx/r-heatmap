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

    pub fn scan_temps() -> Vec<Reading> {
        let temps = temps_platform();
        reading::found("GPU temperature sensors", &temps);
        temps
    }

    pub fn scan_clocks() -> Vec<Reading> {
        let clocks = clocks_platform();
        reading::found("GPU clock domains", &clocks);
        clocks
    }

    pub fn refresh(gpus: &mut [Reading]) {
        refresh_platform(gpus);
    }

    pub fn refresh_sensors(sensors: &mut [Reading]) {
        reading::refresh(sensors);
    }
}

#[cfg(unix)]
fn scan_platform() -> Vec<Reading> {
    drm::usage()
}

#[cfg(unix)]
fn temps_platform() -> Vec<Reading> {
    drm::temperatures()
}

#[cfg(unix)]
fn clocks_platform() -> Vec<Reading> {
    drm::clocks()
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
fn temps_platform() -> Vec<Reading> {
    Vec::new()
}

#[cfg(windows)]
fn clocks_platform() -> Vec<Reading> {
    Vec::new()
}

#[cfg(windows)]
fn refresh_platform(gpus: &mut [Reading]) {
    reading::update(gpus, &perf::gpu_usage());
}
