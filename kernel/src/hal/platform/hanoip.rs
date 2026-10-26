//! Qualcomm Hanoip (SM7150 / SM6225) Platform HAL Backend Specification
//!
//! Constitutional References:
//! - REQ-HAL-102: Secondary backend architecture target (hanoip)
//! - REQ-HAL-103: Isolated behind standard trait interfaces; no driver cfg sprawl
//! - Rule N-04: Zero unsafe code in new kernel modules

use crate::hal::traits::{
    ClockResetController, DmaAllocator, HalDevice, HalError, IrqController,
    PowerController, PowerState, RegisterAccess,
};

pub const HANOIP_UART_BASE: u64 = 0x0a88_0000; // Qualcomm GENI UART
pub const HANOIP_PDC_GIC_BASE: u64 = 0x17a0_0000; // Qualcomm PDC / GICv3
pub const HANOIP_UFS_BASE: u64 = 0x01d8_4000; // Qualcomm UFS 2.2 Controller
pub const HANOIP_DMA_BASE: u64 = 0x8000_0000; // Qualcomm DDR System Pool

/// Target MMIO register descriptor for Qualcomm Hanoip platform.
#[derive(Debug, Clone)]
pub struct HanoipMmio {
    base_address: u64,
    size_bytes: usize,
}

impl HanoipMmio {
    pub const fn new(base_address: u64, size_bytes: usize) -> Self {
        Self {
            base_address,
            size_bytes,
        }
    }

    pub fn base_address(&self) -> u64 {
        self.base_address
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }
}

impl RegisterAccess for HanoipMmio {
    fn read_u8(&self, _offset: usize) -> Result<u8, HalError> {
        Ok(0)
    }

    fn write_u8(&mut self, _offset: usize, _val: u8) -> Result<(), HalError> {
        Ok(())
    }

    fn read_u32(&self, _offset: usize) -> Result<u32, HalError> {
        Ok(0)
    }

    fn write_u32(&mut self, _offset: usize, _val: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn read_u64(&self, _offset: usize) -> Result<u64, HalError> {
        Ok(0)
    }

    fn write_u64(&mut self, _offset: usize, _val: u64) -> Result<(), HalError> {
        Ok(())
    }

    fn barrier(&self) {}
}

/// Qualcomm PDC / GICv3 interrupt controller stub.
#[derive(Debug, Clone)]
pub struct HanoipIrqController;

impl IrqController for HanoipIrqController {
    fn enable_irq(&mut self, _irq_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn disable_irq(&mut self, _irq_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn ack_irq(&mut self) -> Result<u32, HalError> {
        Ok(0)
    }

    fn eoi_irq(&mut self, _irq_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn register_handler(&mut self, _irq_id: u32, _handler_id: u32) -> Result<(), HalError> {
        Ok(())
    }
}

/// Qualcomm coherent DMA memory allocator stub.
#[derive(Debug, Clone)]
pub struct HanoipDmaAllocator;

impl DmaAllocator for HanoipDmaAllocator {
    fn allocate_coherent(&mut self, size_bytes: usize, _alignment: usize) -> Result<(u64, usize), HalError> {
        Ok((HANOIP_DMA_BASE, size_bytes))
    }

    fn free_coherent(&mut self, _paddr: u64, _size_bytes: usize) -> Result<(), HalError> {
        Ok(())
    }

    fn flush_cache(&self, _paddr: u64, _size_bytes: usize) -> Result<(), HalError> {
        Ok(())
    }

    fn invalidate_cache(&self, _paddr: u64, _size_bytes: usize) -> Result<(), HalError> {
        Ok(())
    }
}

/// Qualcomm GCC (Global Clock Controller) stub.
#[derive(Debug, Clone)]
pub struct HanoipClockReset;

impl ClockResetController for HanoipClockReset {
    fn enable_clock(&mut self, _clock_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn disable_clock(&mut self, _clock_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn assert_reset(&mut self, _reset_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn deassert_reset(&mut self, _reset_id: u32) -> Result<(), HalError> {
        Ok(())
    }

    fn get_frequency_hz(&self, clock_id: u32) -> Result<u64, HalError> {
        match clock_id {
            1 => Ok(19_200_000), // Qualcomm TCXO 19.2 MHz
            _ => Ok(100_000_000),
        }
    }
}

/// Qualcomm RPMh / PSCI power controller stub.
#[derive(Debug, Clone)]
pub struct HanoipPower;

impl PowerController for HanoipPower {
    fn set_domain_power(&mut self, _domain_id: u32, _state: PowerState) -> Result<(), HalError> {
        Ok(())
    }

    fn get_domain_power(&self, _domain_id: u32) -> Result<PowerState, HalError> {
        Ok(PowerState::Active)
    }
}

/// Qualcomm Hanoip device model descriptor.
#[derive(Debug, Clone)]
pub struct HanoipHalDevice {
    device_id: u32,
    name: &'static str,
    power_state: PowerState,
}

impl HanoipHalDevice {
    pub const fn new(device_id: u32, name: &'static str) -> Self {
        Self {
            device_id,
            name,
            power_state: PowerState::Active,
        }
    }
}

impl HalDevice for HanoipHalDevice {
    fn device_id(&self) -> u32 {
        self.device_id
    }

    fn name(&self) -> &str {
        self.name
    }

    fn power_state(&self) -> PowerState {
        self.power_state
    }

    fn set_power_state(&mut self, state: PowerState) -> Result<(), HalError> {
        self.power_state = state;
        Ok(())
    }

    fn capabilities(&self) -> u32 {
        0x01 // Qualcomm hardware acceleration capability
    }
}
