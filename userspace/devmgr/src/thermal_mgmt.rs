//! Thermal Management & Throttling Driver (SP-82, REQ-HW-108, REQ-THM-101)
//!
//! Architectural Invariants:
//! - INV-THM-001: TSENS controller MMIO mapping (0x01f0_8000)
//! - INV-THM-002: Conservative thermal trip-points (45C, 65C, 85C)
//! - INV-THM-003: Dynamic frequency step-down enforcement

pub const HANOIP_TSENS0_BASE: u64 = 0x01f0_8000;
pub const THERMAL_WARN_MILLICELSIUS: i32 = 45_000;
pub const THERMAL_THROTTLE_MILLICELSIUS: i32 = 65_000;
pub const THERMAL_SHUTDOWN_MILLICELSIUS: i32 = 85_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalZone {
    Normal,
    Warning,
    Throttle,
    EmergencyShutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalError {
    InvalidMmioBase,
    CriticalThermalTrip,
}

pub struct ThermalMgmtHarness {
    mmio_mapped: bool,
    current_temp_mc: i32,
    zone: ThermalZone,
    clock_budget_pct: u8,
}

impl ThermalMgmtHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            current_temp_mc: 32_000, // 32.0C baseline
            zone: ThermalZone::Normal,
            clock_budget_pct: 100,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), ThermalError> {
        if base != HANOIP_TSENS0_BASE {
            return Err(ThermalError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn sample_sensor(&mut self, temp_millicelsius: i32) -> Result<ThermalZone, ThermalError> {
        self.current_temp_mc = temp_millicelsius;
        if temp_millicelsius >= THERMAL_SHUTDOWN_MILLICELSIUS {
            self.zone = ThermalZone::EmergencyShutdown;
            self.clock_budget_pct = 0;
            return Err(ThermalError::CriticalThermalTrip);
        } else if temp_millicelsius >= THERMAL_THROTTLE_MILLICELSIUS {
            self.zone = ThermalZone::Throttle;
            self.clock_budget_pct = 50; // Throttle to 50%
        } else if temp_millicelsius >= THERMAL_WARN_MILLICELSIUS {
            self.zone = ThermalZone::Warning;
            self.clock_budget_pct = 80;
        } else {
            self.zone = ThermalZone::Normal;
            self.clock_budget_pct = 100;
        }
        Ok(self.zone)
    }

    pub fn current_temp_mc(&self) -> i32 {
        self.current_temp_mc
    }

    pub fn zone(&self) -> ThermalZone {
        self.zone
    }

    pub fn clock_budget_pct(&self) -> u8 {
        self.clock_budget_pct
    }
}
