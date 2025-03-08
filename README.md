# Antec Flux Pro (AF Pro) Display

This is a service that displays CPU and GPU temperatures on the [Antec Flux Pro](https://www.antec.com/product/case/flux-pro) display. It supports NVIDIA GPUs through NVML and reads CPU temperatures using system sensors.

## Installation

### Using the Debian Package (Recommended)

```bash
# Install the package
sudo dpkg -i af-pro-display_0.1.0_amd64.deb
sudo apt install -f  # Install any missing dependencies

# Log out and log back in for group changes to take effect
```

### Building from Source

1. Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Clone and build the project:
```bash
git clone https://github.com/your-username/af-pro-display.git
cd af-pro-display
cargo build --release
```

3. Build the debian package:
```bash
cargo install cargo-deb  # Only needed once
cargo deb
```

The package will be created in `target/debian/`.

## Service Management

After installation, the service will automatically start. You can manage it using systemd:

```bash
# Check service status
sudo systemctl status af-pro-display

# View service logs
journalctl -u af-pro-display

# Stop the service
sudo systemctl stop af-pro-display

# Start the service
sudo systemctl start af-pro-display

# Disable service autostart
sudo systemctl disable af-pro-display

# Enable service autostart
sudo systemctl enable af-pro-display
```

## Troubleshooting

If you encounter permission errors:

1. Verify the device is connected:
```bash
lsusb | grep "2022:0522"
```

2. Check udev rules are loaded:
```bash
ls -l /dev/bus/usb/$(lsusb | grep "2022:0522" | cut -d' ' -f2,4 | sed 's/:/\//')
```

3. Verify group membership:
```bash
groups | grep plugdev
```

4. Check service logs for errors:
```bash
journalctl -u af-pro-display -n 50 --no-pager
```

## Features

- Real-time CPU temperature monitoring
- NVIDIA GPU temperature support through NVML
- Automatic USB device detection and management
- Systemd service integration

## Resources
* [cargo-deb](https://crates.io/crates/cargo-deb)
* []
* Inspired by previous work written by [AKoskovich](https://github.com/AKoskovich/antec_flux_pro_display_service)

## License

This project is licensed under [GNU GENERAL PUBLIC LICENSE Version 3](LICENSE) - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.