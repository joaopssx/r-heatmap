pub struct GpuMonitor;

impl GpuMonitor {
    pub fn get_usage() -> Option<f32> {
        let paths = [
            "/sys/class/drm/card0/device/gpu_busy_percent",
            "/sys/devices/pci0000:00/0000:00:08.1/0000:0d:00.0/gpu_busy_percent",
        ];

        for path in paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(val) = content.trim().parse::<f32>() {
                    return Some(val);
                }
            }
        }
        None
    }
}
