mod config;
mod cpu;
mod gpu;
mod usb;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, path::PathBuf, time::Duration};

use anyhow::Result;
use clap::Parser;

use config::{Config, FromConfigFile};
use cpu::default_cpu_device;
use gpu::AvailableGpu;
use usb::UsbDevice;

#[derive(clap::Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "~/.config/af-pro-display/config.toml")]
    config: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = shellexpand::tilde(&cli.config).to_string();
    let config_path = PathBuf::from(&config_path);
    if fs::File::open(&config_path).is_err() {
        eprintln!("Config file not found at: {}", config_path.display());
        eprintln!("Creating default config file...");

        let config_dir = config_path.parent().ok_or(anyhow::anyhow!(
            "Failed to get parent directory of config file"
        ))?;
        fs::create_dir_all(config_dir)?;
        fs::write(&config_path, toml::to_string(&Config::default())?)?;
    }

    let running = Arc::new(AtomicBool::new(true));
    let config = Config::from_config_file(&config_path)?;
    let device = UsbDevice::open(usb::VENDOR_ID, usb::PRODUCT_ID)?;
    let cpu = config.cpu_device.or_else(default_cpu_device);
    let gpu = AvailableGpu::get_available_gpu();

    // Handle CTRL+C and other termination gracefully
    let run = running.clone();
    ctrlc::set_handler(move || {
        run.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Loop until the program is terminated
    while running.load(Ordering::SeqCst) {
        let cpu_temp = &cpu.as_ref().and_then(|path| cpu::read_temp(path));
        let gpu_temp = &gpu.temp();

        device.send_payload(cpu_temp, gpu_temp);
        std::thread::sleep(Duration::from_millis(config.polling_interval));
    }

    // Finally, set the temps to zero before exiting
    device.send_payload(&Some(0.0), &Some(0.0));

    Ok(())
}
