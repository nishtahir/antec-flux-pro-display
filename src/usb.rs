use std::time::Duration;

use anyhow::Result;

pub const VENDOR_ID: u16 = 0x2022;
pub const PRODUCT_ID: u16 = 0x0522;

pub struct UsbDevice {
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
}

impl UsbDevice {
    pub fn open(vendor_id: u16, product_id: u16) -> Result<Self> {
        match rusb::open_device_with_vid_pid(vendor_id, product_id) {
            Some(handle) => Ok(Self { handle }),
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

                    if device_desc.vendor_id() == VENDOR_ID
                        && device_desc.product_id() == PRODUCT_ID
                    {
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

    pub fn send_payload(&self, cpu_temp: &Option<f32>, gpu_temp: &Option<f32>) {
        let payload = generate_payload(cpu_temp, gpu_temp);

        let config_desc = match self.handle.device().config_descriptor(0) {
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

        match self
            .handle
            .write_bulk(endpoint_address, &payload, Duration::from_millis(1000))
        {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error writing bulk: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}

fn generate_payload(cpu_temp: &Option<f32>, gpu_temp: &Option<f32>) -> Vec<u8> {
    let mut payload = Vec::<u8>::new();
    payload.push(85);
    payload.push(170);
    payload.push(1);
    payload.push(1);
    payload.push(6);

    let encoded_temp = encode_temperature(cpu_temp);
    payload.push(encoded_temp.0);
    payload.push(encoded_temp.1);
    payload.push(encoded_temp.2);

    let encoded_temp = encode_temperature(gpu_temp);
    payload.push(encoded_temp.0);
    payload.push(encoded_temp.1);
    payload.push(encoded_temp.2);

    let checksum = payload.iter().fold(0u8, |acc, e| acc.wrapping_add(*e));
    payload.push(checksum);
    payload
}

fn encode_temperature(temp: &Option<f32>) -> (u8, u8, u8) {
    if let Some(temp) = temp {
        let ones = (temp / 10.0) as u8;
        let tens = (temp % 10.0) as u8;
        let tenths = ((temp * 10.0) % 10.0) as u8;
        return (ones, tens, tenths);
    }
    (238, 238, 238)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_payload() {
        let actual = generate_payload(&Some(24.0), &Some(16.0));
        let expected = vec![85, 170, 1, 1, 6, 2, 4, 0, 1, 6, 0, 20];
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_generate_payload_with_no_gpu() {
        let actual = generate_payload(&Some(24.0), &None);
        let expected = vec![85, 170, 1, 1, 6, 2, 4, 0, 238, 238, 238, 215];
        assert_eq!(expected, actual);
    }
}
