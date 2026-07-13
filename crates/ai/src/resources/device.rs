use candle_core::{DType, Device};

pub fn default() -> Device {
    Device::Cpu
}

pub fn dtype() -> DType {
    DType::F32
}
