//! Kernel Qualcomm WCN3990 Wi-Fi Driver (SP-87, REQ-HW-109, REQ-NET-101)
//!
//! Architectural Invariants:
//! - INV-WIFI-001: WCN3990 SMD MMIO base mapping (0x0a00_0000)
//! - INV-WIFI-002: Firmware payload handoff and RF calibration
//! - INV-WIFI-003: 802.11 b/g/n/ac association state machine

pub const KERNEL_HANOIP_WCN3990_BASE: u64 = 0x0a00_0000;
pub const KERNEL_CAP_NET_ADMIN: u32 = 0x0200_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelWifiState {
    Disabled,
    Idle,
    Scanning,
    Authenticating,
    Associated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelWifiError {
    InvalidMmioBase,
    FirmwareNotStaged,
    PermissionDenied,
    InvalidStateTransition,
    CalibrationFailed,
}

pub struct KernelWifiWcn3990Harness {
    mmio_mapped: bool,
    calibrated: bool,
    state: KernelWifiState,
}

impl KernelWifiWcn3990Harness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            calibrated: false,
            state: KernelWifiState::Disabled,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), KernelWifiError> {
        if base != KERNEL_HANOIP_WCN3990_BASE {
            return Err(KernelWifiError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn calibrate_and_init(&mut self, fw_staged: bool) -> Result<(), KernelWifiError> {
        if !self.mmio_mapped {
            return Err(KernelWifiError::InvalidMmioBase);
        }
        if !fw_staged {
            return Err(KernelWifiError::FirmwareNotStaged);
        }
        self.calibrated = true;
        self.state = KernelWifiState::Idle;
        Ok(())
    }

    pub fn start_scan(&mut self, cap_mask: u32) -> Result<(), KernelWifiError> {
        if (cap_mask & KERNEL_CAP_NET_ADMIN) == 0 {
            return Err(KernelWifiError::PermissionDenied);
        }
        if self.state != KernelWifiState::Idle && self.state != KernelWifiState::Associated {
            return Err(KernelWifiError::InvalidStateTransition);
        }
        self.state = KernelWifiState::Scanning;
        Ok(())
    }

    pub fn associate(&mut self, cap_mask: u32, ssid: &[u8]) -> Result<(), KernelWifiError> {
        if (cap_mask & KERNEL_CAP_NET_ADMIN) == 0 {
            return Err(KernelWifiError::PermissionDenied);
        }
        if ssid.is_empty() {
            return Err(KernelWifiError::InvalidStateTransition);
        }
        self.state = KernelWifiState::Authenticating;
        self.state = KernelWifiState::Associated;
        Ok(())
    }

    pub fn state(&self) -> KernelWifiState {
        self.state
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_calibrated(&self) -> bool {
        self.calibrated
    }
}
