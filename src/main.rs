mod cpu;
mod gpu;
mod usb;

use anyhow::Result;
use gpu::AvailableGpu;
use std::time::Duration;
use usb::UsbDevice;

fn main() -> Result<()> {
    let device = UsbDevice::open(usb::VENDOR_ID, usb::PRODUCT_ID)?;
    let gpu = AvailableGpu::get_available_gpu();
    loop {
        let cpu_temp = cpu::get_cpu_temp();
        let gpu_temp = gpu.temp();

        device.send_payload(cpu_temp, gpu_temp);
        std::thread::sleep(Duration::from_millis(200));
    }
}
