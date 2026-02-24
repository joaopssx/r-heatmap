use sysinfo::{CpuRefreshKind, System};

pub struct CpuMonitor;

impl CpuMonitor {
    pub fn refresh(sys: &mut System) {
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    }

    pub fn get_global_usage(sys: &System) -> f32 {
        sys.global_cpu_usage()
    }

    pub fn get_cores_usage(sys: &System) -> Vec<f32> {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
    }
}
