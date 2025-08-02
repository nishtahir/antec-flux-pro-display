use super::{Gpu, GpuProvider};
use anyhow::{Context, Result};
use nvml_wrapper::Device;
use nvml_wrapper::{Nvml, enum_wrappers::device::TemperatureSensor};
use std::sync::OnceLock;

pub struct NvidiaProvider();

static NVML_INSTANCE: OnceLock<Option<Nvml>> = OnceLock::new();

impl GpuProvider for NvidiaProvider {
    fn list() -> Vec<std::boxed::Box<dyn Gpu>> {
        let Some(nvml) = NVML_INSTANCE.get_or_init(|| {
            Nvml::builder()
                .lib_path(std::ffi::OsStr::new("libnvidia-ml.so.1"))
                .init()
                .context("Failed to initialize NVML")
                .ok()
        }) else {
            println!("Could not get a nvml valide instance");
            return vec![];
        };

        let Ok(driver_version) = nvml
            .sys_driver_version()
            .context("Failed to get NVML driver version")
        else {
            return vec![];
        };

        println!("NVML initialized, driver version: {driver_version}");
        let Ok(device_count) = nvml
            .device_count()
            .context("Failed to get NVML device count")
        else {
            return vec![];
        };

        println!("Found {device_count} NVML-supported GPUs");
        (0..device_count)
            .map(|i| nvml.device_by_index(i))
            .filter_map(|d| d.map(|dev| Box::new(NvidiaGpu(dev)) as Box<dyn Gpu>).ok())
            .collect()
    }
}

pub struct NvidiaGpu(Device<'static>);

impl Gpu for NvidiaGpu {
    fn brand(&self) -> &'static str {
        "Nvidia"
    }

    fn name(&self) -> String {
        match self.0.name() {
            Ok(x) => x,
            Err(_) => "unknown".to_string(),
        }
    }

    fn path(&self) -> String {
        /// XXX: this is obviously not the path but I do not have a nvidia gpu
        let Ok(info) = self.0.pci_info() else {
            return "unknown".to_string();
        };
        info.bus_id
    }

    fn temp(&self) -> Result<f32> {
        Ok(self
            .0
            .temperature(TemperatureSensor::Gpu)
            .inspect_err(|e| eprintln!("Error getting Nvidia GPU temperature: {e:?}"))
            .map(|temp| temp as f32)?)
    }
}
