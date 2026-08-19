// Core system monitoring logic - by joaopssx
pub mod cpu;
pub mod disk;
#[cfg(unix)]
pub mod drm;
pub mod fan;
pub mod gpu;
#[cfg(unix)]
pub mod hwmon;
#[cfg(unix)]
pub mod meminfo;
pub mod memory;
pub mod reading;
#[cfg(unix)]
pub mod stat;
#[cfg(unix)]
pub mod sysfs;
pub mod volt;
#[cfg(windows)]
pub mod windows;

pub use cpu::CpuMonitor;
pub use disk::DiskMonitor;
pub use fan::FanMonitor;
pub use gpu::GpuMonitor;
pub use memory::MemoryMonitor;
pub use reading::Reading;
use sysinfo::{Components, System};
pub use volt::VoltMonitor;

pub struct SystemStats {
    pub sys: System,
    pub components: Components,
    pub cores: Vec<Reading>,
    pub memory: Vec<Reading>,
    pub disks: Vec<Reading>,
    pub fans: Vec<Reading>,
    pub volts: Vec<Reading>,
    pub gpus: Vec<Reading>,
    pub gpu_temps: Vec<Reading>,
    pub gpu_clocks: Vec<Reading>,
}

impl SystemStats {
    pub fn new(gpu_enabled: bool) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let components = Components::new_with_refreshed_list();
        let cores = CpuMonitor::scan(&sys);

        Self {
            sys,
            components,
            cores,
            memory: MemoryMonitor::scan(),
            disks: DiskMonitor::scan(),
            fans: FanMonitor::scan(),
            volts: VoltMonitor::scan(),
            gpus: if gpu_enabled {
                GpuMonitor::scan()
            } else {
                Vec::new()
            },
            gpu_temps: if gpu_enabled {
                GpuMonitor::scan_temps()
            } else {
                Vec::new()
            },
            gpu_clocks: if gpu_enabled {
                GpuMonitor::scan_clocks()
            } else {
                Vec::new()
            },
        }
    }

    pub fn refresh(&mut self) {
        CpuMonitor::refresh(&mut self.sys);
        CpuMonitor::refresh_cores(&mut self.cores, &self.sys);
        MemoryMonitor::refresh(&mut self.sys);
        MemoryMonitor::refresh_pools(&mut self.memory);
        DiskMonitor::refresh(&mut self.disks);
        FanMonitor::refresh(&mut self.fans);
        VoltMonitor::refresh(&mut self.volts);
        GpuMonitor::refresh(&mut self.gpus);
        GpuMonitor::refresh_sensors(&mut self.gpu_temps);
        GpuMonitor::refresh_sensors(&mut self.gpu_clocks);
        self.components.refresh(true);
    }

    pub fn cpu_usage(&self) -> f32 {
        CpuMonitor::get_global_usage(&self.sys)
    }

    pub fn mem_usage(&self) -> (u64, u64) {
        MemoryMonitor::get_usage(&self.sys)
    }
}
