use nvml_wrapper::{Nvml, enum_wrappers::device::TemperatureSensor};
use std::time::Duration;
use systemstat::{Platform, System};

const VENDOR_ID: u16 = 0x2022;
const PRODUCT_ID: u16 = 0x0522;

enum AvailableGpu {
    Nvidia(Nvml),
    Unknown,
}

#[derive(Debug)]
pub enum UsbError {
    DeviceNotFound,
    AccessDenied,
    IoError(rusb::Error),
}

fn get_available_gpu() -> AvailableGpu {
    let nvml = Nvml::init();
    match nvml {
        Ok(nvml) => AvailableGpu::Nvidia(nvml),
        Err(_) => AvailableGpu::Unknown,
    }
}

fn open_usb_device() -> rusb::DeviceHandle<rusb::GlobalContext> {
    match rusb::open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID) {
        Some(handle) => handle,
        None => {
            // Check if device is visible at all
            let devices = match rusb::devices() {
                Ok(devices) => devices,
                Err(e) => {
                    eprintln!("Error getting devices: {}", e);
                    std::process::exit(1);
                }
            };

            for device in devices.iter() {
                let device_desc = match device.device_descriptor() {
                    Ok(desc) => desc,
                    Err(e) => {
                        eprintln!("Error getting device descriptor: {}", e);
                        std::process::exit(1);
                    }
                };

                if device_desc.vendor_id() == VENDOR_ID && device_desc.product_id() == PRODUCT_ID {
                    eprintln!("Permission denied accessing USB device.");
                    eprintln!("Please ensure udev rules are properly configured.");
                    std::process::exit(1);
                }
            }
            eprintln!("USB device not found. Is it connected?");
            eprintln!("Looking for device {:04x}:{:04x}", VENDOR_ID, PRODUCT_ID);
            std::process::exit(1);
        }
    }
}

fn encode_temperature(temp: f32, payload: &mut Vec<u8>) {
    let ones = (temp / 10.0) as u8;
    let tens = (temp % 10.0) as u8;
    let tenths = ((temp * 10.0) % 10.0) as u8;

    payload.push(ones);
    payload.push(tens);
    payload.push(tenths);
}

fn generate_payload(cpu_temp: f32, gpu_temp: f32) -> Vec<u8> {
    let mut payload = Vec::<u8>::new();
    payload.push(85);
    payload.push(170);
    payload.push(1);
    payload.push(1);
    payload.push(6);

    encode_temperature(cpu_temp, &mut payload);
    encode_temperature(gpu_temp, &mut payload);

    let checksum = payload.iter().fold(0u8, |acc, e| acc.wrapping_add(*e));
    payload.push(checksum);
    payload
}

fn get_cpu_temp() -> f32 {
    let sys = System::new();
    match sys.cpu_temp() {
        Ok(temp) => temp,
        Err(e) => {
            eprintln!("Error getting CPU temp: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_gpu_temp(gpu: &AvailableGpu) -> f32 {
    match gpu {
        AvailableGpu::Nvidia(nvml) => {
            let device = nvml.device_by_index(0).unwrap();
            match device.temperature(TemperatureSensor::Gpu) {
                Ok(temp) => temp as f32,
                Err(e) => {
                    eprintln!("Error getting GPU temp: {}", e);
                    0.0
                }
            }
        }
        AvailableGpu::Unknown => 0.0,
    }
}

fn main() {
    let handle = open_usb_device();
    let gpu = get_available_gpu();
    loop {
        let cpu_temp = get_cpu_temp();
        let gpu_temp = get_gpu_temp(&gpu);
        let payload = generate_payload(cpu_temp, gpu_temp);
        let config_desc = match handle.device().config_descriptor(0) {
            Ok(desc) => desc,
            Err(e) => {
                eprintln!("Error getting config descriptor: {}", e);
                std::process::exit(1);
            }
        };

        // Find the first bulk OUT endpoint
        let endpoint_address = config_desc
            .interfaces()
            .flat_map(|interface| interface.descriptors())
            .flat_map(|desc| desc.endpoint_descriptors())
            .find(|endpoint| {
                endpoint.transfer_type() == rusb::TransferType::Interrupt
                    && endpoint.direction() == rusb::Direction::Out
            })
            .map(|endpoint| endpoint.address())
            // This appears to be the correct endpoint on my machine
            // Seems reasonable as the default
            .unwrap_or(0x03);

        match handle.write_bulk(endpoint_address, &payload, Duration::from_millis(1000)) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error writing bulk: {:?}", e);
                std::process::exit(1);
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_payload() {
        let actual = generate_payload(24.0, 16.0);
        let expected = vec![85, 170, 1, 1, 6, 2, 4, 0, 1, 6, 0, 20];
        assert_eq!(expected, actual);
    }
}
