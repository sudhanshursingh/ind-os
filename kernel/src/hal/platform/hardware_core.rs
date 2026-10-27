//! Kernel Core on Hardware Verification (SP-73, REQ-HW-104, REQ-KRN-101)
//!
//! Architectural Invariants:
//! - INV-CORE-HW-001: GENI UART hardware transport (0x0a88_0000)
//! - INV-CORE-HW-002: PDC / GICv3 interrupt subsystem (0x17a0_0000)
//! - INV-CORE-HW-003: Device Tree DRAM memory mapping (0x8000_0000, 6GB)

pub const KERNEL_HANOIP_GENI_UART_BASE: u64 = 0x0a88_0000;
pub const KERNEL_HANOIP_PDC_GIC_BASE: u64 = 0x17a0_0000;
pub const KERNEL_HANOIP_DRAM_BASE: u64 = 0x8000_0000;
pub const KERNEL_HANOIP_DRAM_SIZE_BYTES: u64 = 6 * 1024 * 1024 * 1024; // 6GB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelHardwareCoreError {
    InvalidBaseAddress,
    MemoryRangeOverflow,
    InterruptRoutingFailed,
}

pub struct KernelHardwareCoreHarness {
    uart_initialized: bool,
    gic_initialized: bool,
    dram_mapped: bool,
}

impl KernelHardwareCoreHarness {
    pub const fn new() -> Self {
        Self {
            uart_initialized: false,
            gic_initialized: false,
            dram_mapped: false,
        }
    }

    pub fn initialize_uart(&mut self, base: u64) -> Result<(), KernelHardwareCoreError> {
        if base != KERNEL_HANOIP_GENI_UART_BASE {
            return Err(KernelHardwareCoreError::InvalidBaseAddress);
        }
        self.uart_initialized = true;
        Ok(())
    }

    pub fn initialize_gic(&mut self, base: u64) -> Result<(), KernelHardwareCoreError> {
        if base != KERNEL_HANOIP_PDC_GIC_BASE {
            return Err(KernelHardwareCoreError::InvalidBaseAddress);
        }
        self.gic_initialized = true;
        Ok(())
    }

    pub fn map_dram(&mut self, base: u64, size: u64) -> Result<(), KernelHardwareCoreError> {
        if base != KERNEL_HANOIP_DRAM_BASE {
            return Err(KernelHardwareCoreError::InvalidBaseAddress);
        }
        if size > KERNEL_HANOIP_DRAM_SIZE_BYTES {
            return Err(KernelHardwareCoreError::MemoryRangeOverflow);
        }
        self.dram_mapped = true;
        Ok(())
    }

    pub fn uart_initialized(&self) -> bool {
        self.uart_initialized
    }

    pub fn gic_initialized(&self) -> bool {
        self.gic_initialized
    }

    pub fn dram_mapped(&self) -> bool {
        self.dram_mapped
    }
}
