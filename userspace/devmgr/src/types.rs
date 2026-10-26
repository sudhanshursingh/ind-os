//! Ind OS Device Manager Core Types
//!
//! Standards:
//! - REQ-USE-203: Device manager + hotplug events: capability handout
//! - INV-DEV-001: Zero ambient device authority
//! - INV-DEV-002: Hotplug lifecycle & atomic revocation

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusType {
    Platform,
    VirtIO,
    PciExpress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Block,
    Network,
    Console,
    Input,
    Gpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId {
    pub bus_type: BusType,
    pub bus_index: u32,
    pub device_type: DeviceType,
    pub instance_id: u32,
}

impl DeviceId {
    pub const fn new(bus_type: BusType, bus_index: u32, device_type: DeviceType, instance_id: u32) -> Self {
        Self {
            bus_type,
            bus_index,
            device_type,
            instance_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    Unattached,
    Attached,
    Bound,
    Departed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareDescriptor {
    pub mmio_base: usize,
    pub mmio_size: usize,
    pub irq_line: u32,
    pub dma_buffer_size: usize,
}

impl HardwareDescriptor {
    pub const fn new(mmio_base: usize, mmio_size: usize, irq_line: u32, dma_buffer_size: usize) -> Self {
        Self {
            mmio_base,
            mmio_size,
            irq_line,
            dma_buffer_size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceError {
    DeviceNotFound,
    AlreadyAttached,
    AccessDenied,
    DeviceDeparted,
    BufferFull,
    DriverMismatch,
}
