//! Ind OS Hardware Abstraction Layer (HAL) Traits
//!
//! Constitutional References:
//! - REQ-HAL-101: Generic device-model trait defined (MMIO, IRQ, DMA, power, errors)
//! - REQ-HAL-103: Backend-agnostic driver interface without cfg sprawl
//! - REQ-HAL-105: Interface stability protocol

/// Operational power states for HAL managed devices and power domains.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    Active = 0,
    Standby = 1,
    Retention = 2,
    PoweredOff = 3,
}

/// Standardized error codes across all HAL platform backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalError {
    DeviceNotFound,
    InvalidOffset,
    InvalidIrq,
    DmaOutOfMemory,
    DmaAlignmentError,
    ClockDisabled,
    UnsupportedPowerState,
    HardwareFault,
    PermissionDenied,
}

/// Memory-mapped I/O register access interface with barrier semantics.
pub trait RegisterAccess {
    fn read_u8(&self, offset: usize) -> Result<u8, HalError>;
    fn write_u8(&mut self, offset: usize, val: u8) -> Result<(), HalError>;
    fn read_u32(&self, offset: usize) -> Result<u32, HalError>;
    fn write_u32(&mut self, offset: usize, val: u32) -> Result<(), HalError>;
    fn read_u64(&self, offset: usize) -> Result<u64, HalError>;
    fn write_u64(&mut self, offset: usize, val: u64) -> Result<(), HalError>;
    fn barrier(&self);
}

/// Interrupt controller abstraction for line routing, masking, and EOI.
pub trait IrqController {
    fn enable_irq(&mut self, irq_id: u32) -> Result<(), HalError>;
    fn disable_irq(&mut self, irq_id: u32) -> Result<(), HalError>;
    fn ack_irq(&mut self) -> Result<u32, HalError>;
    fn eoi_irq(&mut self, irq_id: u32) -> Result<(), HalError>;
    fn register_handler(&mut self, irq_id: u32, handler_id: u32) -> Result<(), HalError>;
}

/// Coherent and streaming DMA allocation and cache maintenance interface.
pub trait DmaAllocator {
    fn allocate_coherent(&mut self, size_bytes: usize, alignment: usize) -> Result<(u64, usize), HalError>;
    fn free_coherent(&mut self, paddr: u64, size_bytes: usize) -> Result<(), HalError>;
    fn flush_cache(&self, paddr: u64, size_bytes: usize) -> Result<(), HalError>;
    fn invalidate_cache(&self, paddr: u64, size_bytes: usize) -> Result<(), HalError>;
}

/// Clock gate and reset controller interface for platform peripherals.
pub trait ClockResetController {
    fn enable_clock(&mut self, clock_id: u32) -> Result<(), HalError>;
    fn disable_clock(&mut self, clock_id: u32) -> Result<(), HalError>;
    fn assert_reset(&mut self, reset_id: u32) -> Result<(), HalError>;
    fn deassert_reset(&mut self, reset_id: u32) -> Result<(), HalError>;
    fn get_frequency_hz(&self, clock_id: u32) -> Result<u64, HalError>;
}

/// Power domain and device power controller.
pub trait PowerController {
    fn set_domain_power(&mut self, domain_id: u32, state: PowerState) -> Result<(), HalError>;
    fn get_domain_power(&self, domain_id: u32) -> Result<PowerState, HalError>;
}

/// Generic device abstraction encapsulating identification, power, and capabilities.
pub trait HalDevice {
    fn device_id(&self) -> u32;
    fn name(&self) -> &str;
    fn power_state(&self) -> PowerState;
    fn set_power_state(&mut self, state: PowerState) -> Result<(), HalError>;
    fn capabilities(&self) -> u32;
}
