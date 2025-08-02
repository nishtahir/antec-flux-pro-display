use super::{Gpu, GpuProvider};
use anyhow::{Result, anyhow};
use libamdgpu_top::AMDGPU::{GpuMetrics, MetricsInfo};
use libamdgpu_top::DevicePath;
use std::boxed::Box;

pub struct AmdProvider();

impl GpuProvider for AmdProvider {
    fn list() -> Vec<Box<dyn Gpu>> {
        DevicePath::get_device_path_list()
            .into_iter()
            .map(|x| Box::new(AmdGpu(x)) as Box<dyn Gpu>)
            .collect()
    }
}

struct AmdGpu(DevicePath);

impl Gpu for AmdGpu {
    fn brand(&self) -> &'static str {
        "Amd"
    }

    fn name(&self) -> String {
        self.0.device_name.to_string()
    }

    fn path(&self) -> String {
        self.0
            .sysfs_path
            .as_os_str()
            .to_str()
            .unwrap_or("unknown")
            .to_string()
    }

    fn temp(&self) -> Result<f32> {
        let gpu_metrics = GpuMetrics::get_from_sysfs_path(self.0.sysfs_path.clone())
            .inspect_err(|e| eprintln!("Error getting AMD GPU metrics: {:?}", e))?;

        let temp: Vec<u16> = gpu_metrics
            .get_average_temperature_core() // core is in milliCelsius
            .or_else(|| gpu_metrics.get_temperature_hotspot().map(|x| vec![x * 100])) // hotspot is in Celsius
            .ok_or(anyhow!("No temperature core or hotspot found"))?;

        Ok(temp[0] as f32 / 100.0)
    }
}
