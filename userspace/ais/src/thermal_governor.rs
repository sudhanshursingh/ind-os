//! Userspace AIS Thermal Governor & Duty-Cycle Arbiter (SP-97, REQ-AI-102, REQ-THM-102)
//!
//! Dynamic thermal throttling and duty cycle regulation for on-device inference.

pub const AIS_TEMP_NORMAL_MC: u32 = 38_000;
pub const AIS_TEMP_THROTTLE_MC: u32 = 42_000;
pub const AIS_TEMP_EMERGENCY_HALT_MC: u32 = 46_000;
pub const AIS_TEMP_RECOVERY_MC: u32 = 39_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AisThermalState {
    Nominal,
    Throttled,
    EmergencyHalt,
}

pub struct AisThermalGovernorHarness {
    state: AisThermalState,
    current_temp_mc: u32,
    duty_cycle_percent: u8,
}

impl AisThermalGovernorHarness {
    pub const fn new() -> Self {
        Self {
            state: AisThermalState::Nominal,
            current_temp_mc: AIS_TEMP_NORMAL_MC,
            duty_cycle_percent: 100,
        }
    }

    pub fn update_temperature(&mut self, temp_mc: u32) -> AisThermalState {
        self.current_temp_mc = temp_mc;
        if temp_mc >= AIS_TEMP_EMERGENCY_HALT_MC {
            self.state = AisThermalState::EmergencyHalt;
            self.duty_cycle_percent = 0;
        } else if temp_mc >= AIS_TEMP_THROTTLE_MC {
            self.state = AisThermalState::Throttled;
            self.duty_cycle_percent = 50;
        } else if temp_mc <= AIS_TEMP_RECOVERY_MC {
            self.state = AisThermalState::Nominal;
            self.duty_cycle_percent = 100;
        }
        self.state
    }

    pub fn current_duty_cycle_percent(&self) -> u8 {
        self.duty_cycle_percent
    }

    pub fn is_background_allowed(&self) -> bool {
        self.state == AisThermalState::Nominal
    }

    pub fn is_emergency_halted(&self) -> bool {
        self.state == AisThermalState::EmergencyHalt
    }

    pub fn current_temperature_mc(&self) -> u32 {
        self.current_temp_mc
    }

    pub fn state(&self) -> AisThermalState {
        self.state
    }
}
