//! Userspace DevMgr Qualcomm WCN3990 Wi-Fi Driver (SP-87, REQ-HW-109, REQ-NET-101)
//!
//! Architectural Invariants:
//! - INV-WIFI-001: WCN3990 SMD MMIO base mapping (0x0a00_0000)
//! - INV-WIFI-002: Firmware payload handoff and RF calibration
//! - INV-WIFI-003: 802.11 b/g/n/ac association state machine

pub const HANOIP_WCN3990_BASE: u64 = 0x0a00_0000;
pub const CAP_NET_ADMIN: u32 = 0x0200_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    Disabled,
    Idle,
    Scanning,
    Authenticating,
    Associated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiError {
    InvalidMmioBase,
    FirmwareNotStaged,
    PermissionDenied,
    InvalidStateTransition,
    CalibrationFailed,
}

pub struct WifiWcn3990Harness {
    mmio_mapped: bool,
    calibrated: bool,
    state: WifiState,
}

impl WifiWcn3990Harness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            calibrated: false,
            state: WifiState::Disabled,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), WifiError> {
        if base != HANOIP_WCN3990_BASE {
            return Err(WifiError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn calibrate_and_init(&mut self, fw_staged: bool) -> Result<(), WifiError> {
        if !self.mmio_mapped {
            return Err(WifiError::InvalidMmioBase);
        }
        if !fw_staged {
            return Err(WifiError::FirmwareNotStaged);
        }
        self.calibrated = true;
        self.state = WifiState::Idle;
        Ok(())
    }

    pub fn start_scan(&mut self, cap_mask: u32) -> Result<(), WifiError> {
        if (cap_mask & CAP_NET_ADMIN) == 0 {
            return Err(WifiError::PermissionDenied);
        }
        if self.state != WifiState::Idle && self.state != WifiState::Associated {
            return Err(WifiError::InvalidStateTransition);
        }
        self.state = WifiState::Scanning;
        Ok(())
    }

    pub fn associate(&mut self, cap_mask: u32, ssid: &[u8]) -> Result<(), WifiError> {
        if (cap_mask & CAP_NET_ADMIN) == 0 {
            return Err(WifiError::PermissionDenied);
        }
        if ssid.is_empty() {
            return Err(WifiError::InvalidStateTransition);
        }
        self.state = WifiState::Authenticating;
        self.state = WifiState::Associated;
        Ok(())
    }

    pub fn state(&self) -> WifiState {
        self.state
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_calibrated(&self) -> bool {
        self.calibrated
    }
}
