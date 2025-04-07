use crate::cpu::default_cpu_device;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use std::{default::Default, fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub cpu_device: Option<String>,
    pub gpu_device: Option<String>,
    pub polling_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cpu_device: default_cpu_device(),
            gpu_device: None,
            polling_interval: 200,
        }
    }
}

pub trait FromConfigFile {
    /// Load ourselves from the configuration file located at @path
    fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
}

impl<T: DeserializeOwned> FromConfigFile for T {
    fn from_config_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = fs::read_to_string(path)?;
        Ok(toml::from_str(&config)?)
    }
}
