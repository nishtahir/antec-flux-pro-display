use anyhow::Result;

#[cfg(feature = "amd")]
mod amd;
#[cfg(feature = "nvidia")]
mod nvidia;

trait GpuProvider {
    fn list() -> Vec<std::boxed::Box<dyn Gpu>>;
}

pub trait Gpu {
    fn name(&self) -> String;

    fn path(&self) -> String;

    fn temp(&self) -> Result<f32>;

    fn brand(&self) -> &'static str;
}

pub fn get_available_gpus() -> Vec<Box<dyn Gpu>> {
    let mut output = vec![];

    #[cfg(feature = "amd")]
    output.append(&mut amd::AmdProvider::list());

    #[cfg(feature = "nvidia")]
    output.append(&mut nvidia::NvidiaProvider::list());

    output
}
