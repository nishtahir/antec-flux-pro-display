use anyhow::{Context, Result};
use libamdgpu_top::{
    AMDGPU::{GpuMetrics, MetricsInfo},
    DevicePath,
};
use nvml_wrapper::{Nvml, enum_wrappers::device::TemperatureSensor};

pub struct NvidiaGpu {
    nvml: Nvml,
    device_index: u32,
}

impl NvidiaGpu {
    pub fn new(nvml: Nvml) -> Self {
        Self {
            nvml,
            device_index: 0,
        }
    }

    pub fn temp(&self) -> Option<f32> {
        self.nvml
            .device_by_index(self.device_index)
            .inspect_err(|e| eprintln!("Error getting Nvidia GPU device: {:?}", e))
            .and_then(|device| device.temperature(TemperatureSensor::Gpu))
            .inspect_err(|e| eprintln!("Error getting Nvidia GPU temperature: {:?}", e))
            .map(|temp| temp as f32)
            .ok()
    }
}

pub struct AmdGpu {
    device_path: DevicePath,
}

impl AmdGpu {
    pub fn new(device_path: DevicePath) -> Self {
        Self { device_path }
    }

    pub fn temp(&self) -> Option<f32> {
        let gpu_metrics = GpuMetrics::get_from_sysfs_path(self.device_path.sysfs_path.clone())
            .inspect_err(|e| eprintln!("Error getting AMD GPU metrics: {:?}", e))
            .ok()?;
        let temperature: Option<Vec<u16>> = gpu_metrics.get_average_temperature_core();
        temperature.map(|temp| temp[0] as f32 / 100.0)
    }
}

pub enum AvailableGpu {
    Nvidia(NvidiaGpu),
    Amd(AmdGpu),
    Unknown,
}

impl AvailableGpu {
    pub fn get_available_gpu() -> AvailableGpu {
        let maybe_nvidia = try_get_nvidia_gpu()
            .inspect_err(|e| eprintln!("Failed to get Nvidia GPU. Error: {}", e));

        if let Ok(gpu) = maybe_nvidia {
            return gpu;
        }

        let maybe_amd =
            try_get_amdgpu_gpu().inspect_err(|e| eprintln!("Failed to get AMD GPU. Error: {}", e));

        if let Ok(gpu) = maybe_amd {
            return gpu;
        }

        AvailableGpu::Unknown
    }

    pub fn temp(&self) -> Option<f32> {
        match self {
            AvailableGpu::Nvidia(gpu) => gpu.temp(),
            AvailableGpu::Amd(gpu) => gpu.temp(),
            AvailableGpu::Unknown => None,
        }
    }
}

fn try_get_nvidia_gpu() -> Result<AvailableGpu> {
    let nvml = Nvml::builder()
        .lib_path(std::ffi::OsStr::new("libnvidia-ml.so.1"))
        .init()
        .context("Failed to initialize NVML")?;

    let driver_version = nvml
        .sys_driver_version()
        .context("Failed to get NVML driver version")?;
    println!("NVML initialized, driver version: {}", driver_version);

    let device_count = nvml
        .device_count()
        .context("Failed to get NVML device count")?;

    println!("Found {} NVML-supported GPUs", device_count);
    Ok(AvailableGpu::Nvidia(NvidiaGpu::new(nvml)))
}

fn try_get_amdgpu_gpu() -> Result<AvailableGpu> {
    let device_path = DevicePath::get_device_path_list();

    if device_path.is_empty() {
        return Err(anyhow::anyhow!("No AMD GPU devices found"));
    }

    println!("Found {} AMD GPU devices", device_path.len());
    Ok(AvailableGpu::Amd(AmdGpu::new(device_path[0].clone())))
}
