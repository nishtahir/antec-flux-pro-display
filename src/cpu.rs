use std::{fs, str::FromStr};

pub fn read_temp(device: &str) -> Option<f32> {
    fs::read_to_string(device)
        .inspect_err(|e| eprintln!("Error getting CPU temp: {e}"))
        .map(|content| content.trim().to_string())
        .map(|s| f32::from_str(&s).unwrap())
        .map(|num| num / 1000.0)
        .ok()
}

pub fn default_cpu_device() -> Option<String> {
    // Loop through all hwmon devices, find the one that matches CPU temp with the temp1_label that matches Tctl
    // This is common for AMD CPUs where Tctl is used to represent the CPU temperature.
    for hwmon in fs::read_dir("/sys/class/hwmon").ok()? {
        let path = hwmon.ok()?.path();
        if let Ok(label) = fs::read_to_string(format!("{}/temp1_label", path.display())) {
            if label.trim() == "Tctl" {
                return Some(format!("{}/temp1_input", path.display()));
            }
        }
    }

    if fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").is_ok() {
        return Some("/sys/class/thermal/thermal_zone0/temp".to_string());
    }

    if fs::read_to_string("/sys/class/hwmon/hwmon0/temp1_input").is_ok() {
        return Some("/sys/class/hwmon/hwmon0/temp1_input".to_string());
    }

    eprintln!("Could not find CPU temp path");
    None
}
