//! Hardware Lab Preparation & Reference Dev Board Harness (SP-70, REQ-HW-101, REQ-SEC-111)
//!
//! Architectural Invariants:
//! - INV-LAB-001: Serial console transport & baud integrity (115200 8N1)
//! - INV-LAB-002: Power & thermal envelope bounding (3.6V - 4.4V, <= 45.0 C)
//! - INV-LAB-003: Recovery workstation validation (Fastboot / EDL handshake)

pub const SAFE_VOLTAGE_MIN_MV: u32 = 3600; // 3.60V
pub const SAFE_VOLTAGE_MAX_MV: u32 = 4400; // 4.40V
pub const THERMAL_CUTOFF_MILLICELSIUS: u32 = 45000; // 45.0 C
pub const EXPECTED_UART_BAUD: u32 = 115200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabPrepError {
    VoltageOutOfRange,
    ThermalThresholdExceeded,
    BaudRateMismatch,
    RecoveryStationUnreachable,
}

pub struct HardwareLabHarness {
    serial_baud: u32,
    voltage_mv: u32,
    temp_millicelsius: u32,
    recovery_station_verified: bool,
}

impl HardwareLabHarness {
    pub const fn new() -> Self {
        Self {
            serial_baud: 0,
            voltage_mv: 0,
            temp_millicelsius: 0,
            recovery_station_verified: false,
        }
    }

    pub fn configure_serial(&mut self, baud: u32) -> Result<(), LabPrepError> {
        if baud != EXPECTED_UART_BAUD {
            return Err(LabPrepError::BaudRateMismatch);
        }
        self.serial_baud = baud;
        Ok(())
    }

    pub fn audit_power_envelope(&mut self, voltage_mv: u32, temp_millicelsius: u32) -> Result<(), LabPrepError> {
        if voltage_mv < SAFE_VOLTAGE_MIN_MV || voltage_mv > SAFE_VOLTAGE_MAX_MV {
            return Err(LabPrepError::VoltageOutOfRange);
        }
        if temp_millicelsius > THERMAL_CUTOFF_MILLICELSIUS {
            return Err(LabPrepError::ThermalThresholdExceeded);
        }
        self.voltage_mv = voltage_mv;
        self.temp_millicelsius = temp_millicelsius;
        Ok(())
    }

    pub fn verify_recovery_station(&mut self, handshake_ok: bool) -> Result<(), LabPrepError> {
        if !handshake_ok {
            return Err(LabPrepError::RecoveryStationUnreachable);
        }
        self.recovery_station_verified = true;
        Ok(())
    }

    pub fn serial_baud(&self) -> u32 {
        self.serial_baud
    }

    pub fn voltage_mv(&self) -> u32 {
        self.voltage_mv
    }

    pub fn temp_millicelsius(&self) -> u32 {
        self.temp_millicelsius
    }

    pub fn recovery_station_verified(&self) -> bool {
        self.recovery_station_verified
    }
}
