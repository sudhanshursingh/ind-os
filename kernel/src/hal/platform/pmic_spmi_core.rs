//! Kernel Qualcomm PMIC & SPMI Bus Driver (SP-80, REQ-HW-108, REQ-PWR-101)
//!
//! Architectural Invariants:
//! - INV-PMIC-001: SPMI host controller MMIO mapping (0x0940_0000)
//! - INV-PMIC-002: Dual-PMIC slave enumeration (PM7150 at SID 0, PM7150L at SID 2)
//! - INV-PMIC-003: Capability-gated power rail telemetry (CAP_POWER_MGMT)

pub const KERNEL_HANOIP_SPMI_BASE: u64 = 0x0940_0000;
pub const KERNEL_PMIC_PM7150_SID: u8 = 0;
pub const KERNEL_PMIC_PM7150L_SID: u8 = 2;
pub const KERNEL_CAP_POWER_MGMT: u32 = 0x0010_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPmicError {
    InvalidMmioBase,
    DeviceNotFound,
    PermissionDenied,
    BusTimeout,
}

pub struct KernelPmicSpmiHarness {
    mmio_mapped: bool,
    pm7150_present: bool,
    pm7150l_present: bool,
    active_regulators: u32,
}

impl KernelPmicSpmiHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            pm7150_present: false,
            pm7150l_present: false,
            active_regulators: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), KernelPmicError> {
        if base != KERNEL_HANOIP_SPMI_BASE {
            return Err(KernelPmicError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn probe_pmic_slaves(&mut self) -> Result<(), KernelPmicError> {
        if !self.mmio_mapped {
            return Err(KernelPmicError::BusTimeout);
        }
        self.pm7150_present = true;
        self.pm7150l_present = true;
        self.active_regulators = 18;
        Ok(())
    }

    pub fn read_regulator_voltage_mv(&self, cap_mask: u32, sid: u8) -> Result<u32, KernelPmicError> {
        if (cap_mask & KERNEL_CAP_POWER_MGMT) == 0 {
            return Err(KernelPmicError::PermissionDenied);
        }
        match sid {
            KERNEL_PMIC_PM7150_SID => Ok(1800),  // 1.8V VDD_IO
            KERNEL_PMIC_PM7150L_SID => Ok(1200), // 1.2V VDD_CORE
            _ => Err(KernelPmicError::DeviceNotFound),
        }
    }

    pub fn is_pm7150_present(&self) -> bool {
        self.pm7150_present
    }

    pub fn is_pm7150l_present(&self) -> bool {
        self.pm7150l_present
    }

    pub fn active_regulators(&self) -> u32 {
        self.active_regulators
    }
}
