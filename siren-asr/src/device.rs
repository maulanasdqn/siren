use candle_core::Device;
use siren_domain::DomainError;

pub fn pick_device() -> Result<Device, DomainError> {
    #[cfg(feature = "metal")]
    {
        if let Ok(device) = Device::new_metal(0) {
            return Ok(device);
        }
    }

    #[cfg(feature = "cuda")]
    {
        if let Ok(device) = Device::new_cuda(0) {
            return Ok(device);
        }
    }

    Ok(Device::Cpu)
}

pub fn label(device: &Device) -> &'static str {
    match device {
        Device::Cpu => "CPU",
        Device::Cuda(_) => "CUDA (GPU)",
        Device::Metal(_) => "Metal (GPU)",
    }
}
