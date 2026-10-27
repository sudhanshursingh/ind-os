//! Kernel AIS Thermal Governor Core (SP-97, REQ-AI-102, REQ-THM-102)
//!
//! Dynamic thermal throttling and duty cycle regulation for on-device inference.

pub const KERNEL_AIS_TEMP_NORMAL_MC: u32 = 38_000;
pub const KERNEL_AIS_TEMP_THROTTLE_MC: u32 = 42_000;
pub const KERNEL_AIS_TEMP_EMERGENCY_HALT_MC: u32 = 46_000;
pub const KERNEL_AIS_TEMP_RECOVERY_MC: u32 = 39_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAisThermalState {
    Nominal,
    Throttled,
    EmergencyHalt,
}

pub struct KernelAisThermalGovernorHarness {
    state: KernelAisThermalState,
    current_temp_mc: u32,
    duty_cycle_percent: u8,
}

impl KernelAisThermalGovernorHarness {
    pub const fn new() -> Self {
        Self {
            state: KernelAisThermalState::Nominal,
            current_temp_mc: KERNEL_AIS_TEMP_NORMAL_MC,
            duty_cycle_percent: 100,
        }
    }

    pub fn update_temperature(&mut self, temp_mc: u32) -> KernelAisThermalState {
        self.current_temp_mc = temp_mc;
        if temp_mc >= KERNEL_AIS_TEMP_EMERGENCY_HALT_MC {
            self.state = KernelAisThermalState::EmergencyHalt;
            self.duty_cycle_percent = 0;
        } else if temp_mc >= KERNEL_AIS_TEMP_THROTTLE_MC {
            self.state = KernelAisThermalState::Throttled;
            self.duty_cycle_percent = 50;
        } else if temp_mc <= KERNEL_AIS_TEMP_RECOVERY_MC {
            self.state = KernelAisThermalState::Nominal;
            self.duty_cycle_percent = 100;
        }
        self.state
    }

    pub fn current_duty_cycle_percent(&self) -> u8 {
        self.duty_cycle_percent
    }

    pub fn is_background_allowed(&self) -> bool {
        self.state == KernelAisThermalState::Nominal
    }

    pub fn is_emergency_halted(&self) -> bool {
        self.state == KernelAisThermalState::EmergencyHalt
    }

    pub fn current_temperature_mc(&self) -> u32 {
        self.current_temp_mc
    }

    pub fn state(&self) -> KernelAisThermalState {
        self.state
    }
}
