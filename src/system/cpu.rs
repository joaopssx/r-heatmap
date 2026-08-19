use crate::system::reading::{self, Reading};
#[cfg(unix)]
use crate::system::{cpufreq, stat};
use sysinfo::{CpuRefreshKind, System};

pub struct CpuMonitor;

impl CpuMonitor {
    pub fn refresh(sys: &mut System) {
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    }

    pub fn get_global_usage(sys: &System) -> f32 {
        sys.global_cpu_usage()
    }

    pub fn scan(sys: &System) -> Vec<Reading> {
        let cores = cores_platform(sys);
        reading::found("CPU cores", &cores);
        cores
    }

    pub fn refresh_cores(cores: &mut [Reading], sys: &System) {
        reading::update(cores, &cores_platform(sys));
    }
}

#[cfg(unix)]
fn cores_platform(_sys: &System) -> Vec<Reading> {
    let mut cores = stat::cores();
    cpufreq::annotate(&mut cores);

    cores
}

#[cfg(not(unix))]
fn cores_platform(sys: &System) -> Vec<Reading> {
    sys.cpus()
        .iter()
        .enumerate()
        .map(|(index, cpu)| Reading::new(format!("Core {index}"), cpu.cpu_usage()))
        .collect()
}
