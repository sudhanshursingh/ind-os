//! Kernel Thermal Management & Throttling Driver (SP-82, REQ-HW-108, REQ-THM-101)
//!
//! Architectural Invariants:
//! - INV-THM-001: TSENS controller MMIO mapping (0x01f0_8000)
//! - INV-THM-002: Conservative thermal trip-points (45C, 65C, 85C)
//! - INV-THM-003: Dynamic frequency step-down enforcement

pub const KERNEL_HANOIP_TSENS0_BASE: u64 = 0x01f0_8000;
pub const KERNEL_THERMAL_WARN_MILLICELSIUS: i32 = 45_000;
pub const KERNEL_THERMAL_THROTTLE_MILLICELSIUS: i32 = 65_000;
pub const KERNEL_THERMAL_SHUTDOWN_MILLICELSIUS: i32 = 85_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelThermalZone {
    Normal,
    Warning,
    Throttle,
    EmergencyShutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelThermalError {
    InvalidMmioBase,
    CriticalThermalTrip,
}

pub struct KernelThermalMgmtHarness {
    mmio_mapped: bool,
    current_temp_mc: i32,
    zone: KernelThermalZone,
    clock_budget_pct: u8,
}

impl KernelThermalMgmtHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            current_temp_mc: 32_000,
            zone: KernelThermalZone::Normal,
            clock_budget_pct: 100,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), KernelThermalError> {
        if base != KERNEL_HANOIP_TSENS0_BASE {
            return Err(KernelThermalError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn sample_sensor(&mut self, temp_millicelsius: i32) -> Result<KernelThermalZone, KernelThermalError> {
        self.current_temp_mc = temp_millicelsius;
        if temp_millicelsius >= KERNEL_THERMAL_SHUTDOWN_MILLICELSIUS {
            self.zone = KernelThermalZone::EmergencyShutdown;
            self.clock_budget_pct = 0;
            return Err(KernelThermalError::CriticalThermalTrip);
        } else if temp_millicelsius >= KERNEL_THERMAL_THROTTLE_MILLICELSIUS {
            self.zone = KernelThermalZone::Throttle;
            self.clock_budget_pct = 50;
        } else if temp_millicelsius >= KERNEL_THERMAL_WARN_MILLICELSIUS {
            self.zone = KernelThermalZone::Warning;
            self.clock_budget_pct = 80;
        } else {
            self.zone = KernelThermalZone::Normal;
            self.clock_budget_pct = 100;
        }
        Ok(self.zone)
    }

    pub fn current_temp_mc(&self) -> i32 {
        self.current_temp_mc
    }

    pub fn zone(&self) -> KernelThermalZone {
        self.zone
    }

    pub fn clock_budget_pct(&self) -> u8 {
        self.clock_budget_pct
    }
}
