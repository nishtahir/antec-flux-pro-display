use std::{fs, str::FromStr};

pub fn read_temp(device: &str) -> Option<f32> {
    fs::read_to_string(device)
        .inspect_err(|e| eprintln!("Error getting CPU temp: {}", e))
        .map(|content| content.trim().to_string())
        .map(|s| f32::from_str(&s).unwrap())
        .map(|num| num / 1000.0)
        .ok()
}

pub fn default_cpu_device() -> Option<String> {
    if fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").is_ok() {
        return Some("/sys/class/thermal/thermal_zone0/temp".to_string());
    }
    if fs::read_to_string("/sys/class/hwmon/hwmon0/temp1_input").is_ok() {
        return Some("/sys/class/hwmon/hwmon0/temp1_input".to_string());
    }

    eprintln!("Could not find CPU temp path");
    None
}
