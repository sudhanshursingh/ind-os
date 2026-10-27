//! Kernel Battery Fuel Gauge & Charging Driver (SP-81, REQ-HW-108, REQ-PWR-102)
//!
//! Architectural Invariants:
//! - INV-BATT-001: Fuel gauge telemetry bounding [0, 100]% & [3200, 4400] mV
//! - INV-BATT-002: Deterministic charging state machine
//! - INV-BATT-003: Critical low-battery emergency shutdown trigger (SoC <= 3%)

pub const KERNEL_BATT_MIN_VOLTAGE_MV: u32 = 3200;
pub const KERNEL_BATT_MAX_VOLTAGE_MV: u32 = 4400;
pub const KERNEL_BATT_CRITICAL_SOC_PCT: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelChargingState {
    Discharging,
    ChargingFast,
    ChargingTrickle,
    Full,
    Fault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelBatteryError {
    VoltageOutOfBounds,
    SocOutOfBounds,
    CriticalBatteryShutdownRequired,
}

pub struct KernelBatteryGaugeHarness {
    soc_pct: u8,
    voltage_mv: u32,
    current_ma: i32,
    state: KernelChargingState,
    emergency_flush_triggered: bool,
}

impl KernelBatteryGaugeHarness {
    pub const fn new() -> Self {
        Self {
            soc_pct: 50,
            voltage_mv: 3850,
            current_ma: -250,
            state: KernelChargingState::Discharging,
            emergency_flush_triggered: false,
        }
    }

    pub fn update_telemetry(&mut self, soc_pct: u8, voltage_mv: u32, current_ma: i32) -> Result<(), KernelBatteryError> {
        if soc_pct > 100 {
            return Err(KernelBatteryError::SocOutOfBounds);
        }
        if voltage_mv < KERNEL_BATT_MIN_VOLTAGE_MV || voltage_mv > KERNEL_BATT_MAX_VOLTAGE_MV {
            return Err(KernelBatteryError::VoltageOutOfBounds);
        }
        self.soc_pct = soc_pct;
        self.voltage_mv = voltage_mv;
        self.current_ma = current_ma;

        if soc_pct <= KERNEL_BATT_CRITICAL_SOC_PCT {
            self.emergency_flush_triggered = true;
            return Err(KernelBatteryError::CriticalBatteryShutdownRequired);
        }

        if current_ma <= 0 {
            self.state = KernelChargingState::Discharging;
        } else if soc_pct == 100 {
            self.state = KernelChargingState::Full;
        } else if soc_pct >= 90 {
            self.state = KernelChargingState::ChargingTrickle;
        } else {
            self.state = KernelChargingState::ChargingFast;
        }
        Ok(())
    }

    pub fn soc_pct(&self) -> u8 {
        self.soc_pct
    }

    pub fn voltage_mv(&self) -> u32 {
        self.voltage_mv
    }

    pub fn state(&self) -> KernelChargingState {
        self.state
    }

    pub fn emergency_flush_triggered(&self) -> bool {
        self.emergency_flush_triggered
    }
}
