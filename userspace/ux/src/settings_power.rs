//! Ind OS Settings & Power Management Hardware Engine (SP-101)
//!
//! Standards: Bible §6.3, §8.15, Table 19.2; REQ-HW-108, REQ-PWR-104
//! Invariants: INV-SETTINGS-PWR-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure memory-safe Rust

pub const CAP_POWER_MGMT: u64 = 0x0000_0000_0000_0080;

pub const BATTERY_FULL_MV: u32 = 4350;
pub const BATTERY_EMPTY_MV: u32 = 3400;
pub const DISPLAY_RATE_ECO_HZ: u32 = 60;
pub const DISPLAY_RATE_PERF_HZ: u32 = 120;

pub const THERMAL_MILD_MC: u32 = 38000;
pub const THERMAL_SEVERE_MC: u32 = 42000;
pub const THERMAL_CRITICAL_MC: u32 = 46000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargingStatus {
    Discharging,
    Charging,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerProfile {
    BatterySaver,
    Balanced,
    Performance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPowerError {
    PermissionDenied,
    InvalidValue,
    DeviceUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryTelemetry {
    pub voltage_mv: u32,
    pub soc_percent: u8,
    pub current_ma: i32,
    pub status: ChargingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerGovernorState {
    pub brightness_pwm: u8,
    pub refresh_rate_hz: u32,
    pub profile: PowerProfile,
    pub is_throttled: bool,
}

pub struct SettingsPowerEngine {
    telemetry: BatteryTelemetry,
    governor: PowerGovernorState,
    initialized: bool,
}

impl SettingsPowerEngine {
    pub const fn new() -> Self {
        Self {
            telemetry: BatteryTelemetry {
                voltage_mv: 4000,
                soc_percent: 88,
                current_ma: -250,
                status: ChargingStatus::Discharging,
            },
            governor: PowerGovernorState {
                brightness_pwm: 180,
                refresh_rate_hz: DISPLAY_RATE_PERF_HZ,
                profile: PowerProfile::Balanced,
                is_throttled: false,
            },
            initialized: false,
        }
    }

    pub fn init(&mut self, caps: u64) -> Result<(), SettingsPowerError> {
        if (caps & CAP_POWER_MGMT) == 0 {
            return Err(SettingsPowerError::PermissionDenied);
        }
        self.initialized = true;
        Ok(())
    }

    pub fn sync_battery(
        &mut self,
        caps: u64,
        voltage_mv: u32,
        soc_percent: u8,
        current_ma: i32,
        status: ChargingStatus,
    ) -> Result<BatteryTelemetry, SettingsPowerError> {
        if (caps & CAP_POWER_MGMT) == 0 {
            return Err(SettingsPowerError::PermissionDenied);
        }
        if soc_percent > 100 {
            return Err(SettingsPowerError::InvalidValue);
        }

        self.telemetry = BatteryTelemetry {
            voltage_mv,
            soc_percent,
            current_ma,
            status,
        };
        Ok(self.telemetry)
    }

    pub fn set_display_governor(
        &mut self,
        caps: u64,
        brightness_pwm: u8,
        refresh_rate_hz: u32,
    ) -> Result<(), SettingsPowerError> {
        if (caps & CAP_POWER_MGMT) == 0 {
            return Err(SettingsPowerError::PermissionDenied);
        }
        if refresh_rate_hz != DISPLAY_RATE_ECO_HZ && refresh_rate_hz != DISPLAY_RATE_PERF_HZ {
            return Err(SettingsPowerError::InvalidValue);
        }

        self.governor.brightness_pwm = brightness_pwm;
        self.governor.refresh_rate_hz = refresh_rate_hz;
        Ok(())
    }

    pub fn apply_thermal_policy(&mut self, temp_mc: u32) -> Result<PowerProfile, SettingsPowerError> {
        if temp_mc >= THERMAL_SEVERE_MC {
            self.governor.profile = PowerProfile::BatterySaver;
            self.governor.refresh_rate_hz = DISPLAY_RATE_ECO_HZ;
            self.governor.is_throttled = true;
            if self.governor.brightness_pwm > 120 {
                self.governor.brightness_pwm = 120;
            }
        } else if temp_mc >= THERMAL_MILD_MC {
            self.governor.profile = PowerProfile::Balanced;
            self.governor.refresh_rate_hz = DISPLAY_RATE_ECO_HZ;
            self.governor.is_throttled = true;
        } else {
            self.governor.profile = PowerProfile::Performance;
            self.governor.refresh_rate_hz = DISPLAY_RATE_PERF_HZ;
            self.governor.is_throttled = false;
        }
        Ok(self.governor.profile)
    }

    pub fn telemetry(&self) -> BatteryTelemetry {
        self.telemetry
    }

    pub fn governor_state(&self) -> PowerGovernorState {
        self.governor
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
