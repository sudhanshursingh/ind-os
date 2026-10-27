//! Kernel Settings & Power Management Hardware Core (SP-101, REQ-HW-108, REQ-PWR-104)
//!
//! Fuel-gauge battery telemetry, display power governor, and PMIC thermal mitigation binding.

pub const KERNEL_CAP_SETTINGS_POWER: u32 = 0x0000_0080;

pub const KERNEL_BATTERY_FULL_MV: u32 = 4350;
pub const KERNEL_BATTERY_EMPTY_MV: u32 = 3400;
pub const KERNEL_DISPLAY_RATE_ECO_HZ: u32 = 60;
pub const KERNEL_DISPLAY_RATE_PERF_HZ: u32 = 120;

pub const KERNEL_THERMAL_MILD_MC: u32 = 38000;
pub const KERNEL_THERMAL_SEVERE_MC: u32 = 42000;
pub const KERNEL_THERMAL_CRITICAL_MC: u32 = 46000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelChargingStatus {
    Discharging,
    Charging,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPowerProfile {
    BatterySaver,
    Balanced,
    Performance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSettingsPowerError {
    PermissionDenied,
    InvalidValue,
    DeviceUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelBatteryTelemetry {
    pub voltage_mv: u32,
    pub soc_percent: u8,
    pub current_ma: i32,
    pub status: KernelChargingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelPowerGovernorState {
    pub brightness_pwm: u8,
    pub refresh_rate_hz: u32,
    pub profile: KernelPowerProfile,
    pub is_throttled: bool,
}

pub struct KernelSettingsPowerHarness {
    telemetry: KernelBatteryTelemetry,
    governor: KernelPowerGovernorState,
    initialized: bool,
}

impl KernelSettingsPowerHarness {
    pub const fn new() -> Self {
        Self {
            telemetry: KernelBatteryTelemetry {
                voltage_mv: 4000,
                soc_percent: 88,
                current_ma: -250,
                status: KernelChargingStatus::Discharging,
            },
            governor: KernelPowerGovernorState {
                brightness_pwm: 180,
                refresh_rate_hz: KERNEL_DISPLAY_RATE_PERF_HZ,
                profile: KernelPowerProfile::Balanced,
                is_throttled: false,
            },
            initialized: false,
        }
    }

    pub fn init(&mut self, cap_token: u32) -> Result<(), KernelSettingsPowerError> {
        if (cap_token & KERNEL_CAP_SETTINGS_POWER) == 0 {
            return Err(KernelSettingsPowerError::PermissionDenied);
        }
        self.initialized = true;
        Ok(())
    }

    pub fn sync_battery(
        &mut self,
        cap_token: u32,
        voltage_mv: u32,
        soc_percent: u8,
        current_ma: i32,
        status: KernelChargingStatus,
    ) -> Result<KernelBatteryTelemetry, KernelSettingsPowerError> {
        if (cap_token & KERNEL_CAP_SETTINGS_POWER) == 0 {
            return Err(KernelSettingsPowerError::PermissionDenied);
        }
        if soc_percent > 100 {
            return Err(KernelSettingsPowerError::InvalidValue);
        }

        self.telemetry = KernelBatteryTelemetry {
            voltage_mv,
            soc_percent,
            current_ma,
            status,
        };
        Ok(self.telemetry)
    }

    pub fn set_display_governor(
        &mut self,
        cap_token: u32,
        brightness_pwm: u8,
        refresh_rate_hz: u32,
    ) -> Result<(), KernelSettingsPowerError> {
        if (cap_token & KERNEL_CAP_SETTINGS_POWER) == 0 {
            return Err(KernelSettingsPowerError::PermissionDenied);
        }
        if refresh_rate_hz != KERNEL_DISPLAY_RATE_ECO_HZ && refresh_rate_hz != KERNEL_DISPLAY_RATE_PERF_HZ {
            return Err(KernelSettingsPowerError::InvalidValue);
        }

        self.governor.brightness_pwm = brightness_pwm;
        self.governor.refresh_rate_hz = refresh_rate_hz;
        Ok(())
    }

    pub fn apply_thermal_policy(&mut self, temp_mc: u32) -> Result<KernelPowerProfile, KernelSettingsPowerError> {
        if temp_mc >= KERNEL_THERMAL_SEVERE_MC {
            self.governor.profile = KernelPowerProfile::BatterySaver;
            self.governor.refresh_rate_hz = KERNEL_DISPLAY_RATE_ECO_HZ;
            self.governor.is_throttled = true;
            if self.governor.brightness_pwm > 120 {
                self.governor.brightness_pwm = 120;
            }
        } else if temp_mc >= KERNEL_THERMAL_MILD_MC {
            self.governor.profile = KernelPowerProfile::Balanced;
            self.governor.refresh_rate_hz = KERNEL_DISPLAY_RATE_ECO_HZ;
            self.governor.is_throttled = true;
        } else {
            self.governor.profile = KernelPowerProfile::Performance;
            self.governor.refresh_rate_hz = KERNEL_DISPLAY_RATE_PERF_HZ;
            self.governor.is_throttled = false;
        }
        Ok(self.governor.profile)
    }

    pub fn telemetry(&self) -> KernelBatteryTelemetry {
        self.telemetry
    }

    pub fn governor_state(&self) -> KernelPowerGovernorState {
        self.governor
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
