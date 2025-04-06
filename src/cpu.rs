use systemstat::{Platform, System};

pub fn get_cpu_temp() -> Option<f32> {
    let sys = System::new();
    sys.cpu_temp()
        .inspect_err(|e| eprintln!("Error getting CPU temp: {}", e))
        .ok()
}
