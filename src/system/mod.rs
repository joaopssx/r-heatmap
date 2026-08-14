// Core system monitoring logic - by joaopssx
pub mod cpu;
pub mod gpu;
pub mod memory;

pub use cpu::CpuMonitor;
pub use gpu::GpuMonitor;
pub use memory::MemoryMonitor;
use sysinfo::{Components, System};

pub struct SystemStats {
    pub sys: System,
    pub components: Components,
    pub gpu_enabled: bool,
}

impl SystemStats {
    pub fn new(gpu_enabled: bool) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let components = Components::new_with_refreshed_list();
        Self {
            sys,
            components,
            gpu_enabled,
        }
    }

    pub fn refresh(&mut self) {
        CpuMonitor::refresh(&mut self.sys);
        MemoryMonitor::refresh(&mut self.sys);
        self.components.refresh(true);
    }

    pub fn cpu_usage(&self) -> f32 {
        CpuMonitor::get_global_usage(&self.sys)
    }

    pub fn cpu_cores_usage(&self) -> Vec<f32> {
        CpuMonitor::get_cores_usage(&self.sys)
    }

    pub fn mem_usage(&self) -> (u64, u64) {
        MemoryMonitor::get_usage(&self.sys)
    }

    pub fn gpu_usage(&self) -> Option<f32> {
        if !self.gpu_enabled {
            return None;
        }
        GpuMonitor::get_usage()
    }
}
