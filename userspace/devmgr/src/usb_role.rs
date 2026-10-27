//! Userspace DevMgr USB Device/Host Role & CDC ACM Console Driver (SP-85, REQ-HW-108, REQ-USB-102, REQ-REC-101)
//!
//! Architectural Invariants:
//! - INV-USB-004: OTG ID pin sensing and role switching
//! - INV-USB-005: USB CDC ACM serial console interface for isolated recovery
//! - INV-USB-006: Capability-gated USB role switching (CAP_USB_ROLE_SWITCH)

pub const USB_ROLE_DEVICE: u8 = 1;
pub const USB_ROLE_HOST: u8 = 2;
pub const CAP_USB_ROLE_SWITCH: u32 = 0x0080_0000;
pub const CDC_ACM_BAUD_DEFAULT: u32 = 115200;
pub const CDC_ACM_LINE_CODING_LEN: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRole {
    Device,
    Host,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRoleError {
    PermissionDenied,
    InvalidRole,
    BufferOverflow,
    ConsoleNotReady,
}

pub struct UsbRoleHarness {
    role: UsbRole,
    baud_rate: u32,
    console_active: bool,
    bytes_transmitted: usize,
}

impl UsbRoleHarness {
    pub const fn new() -> Self {
        Self {
            role: UsbRole::Device,
            baud_rate: CDC_ACM_BAUD_DEFAULT,
            console_active: true,
            bytes_transmitted: 0,
        }
    }

    pub fn sense_otg_id_pin(&mut self, is_grounded: bool) -> UsbRole {
        // Grounded OTG ID pin indicates Host / OTG cable plugged in
        if is_grounded {
            self.role = UsbRole::Host;
        } else {
            self.role = UsbRole::Device;
        }
        self.role
    }

    pub fn switch_role(&mut self, cap_mask: u32, role: UsbRole) -> Result<(), UsbRoleError> {
        if (cap_mask & CAP_USB_ROLE_SWITCH) == 0 {
            return Err(UsbRoleError::PermissionDenied);
        }
        self.role = role;
        Ok(())
    }

    pub fn set_line_coding(&mut self, baud: u32) -> Result<(), UsbRoleError> {
        if baud == 0 {
            return Err(UsbRoleError::InvalidRole);
        }
        self.baud_rate = baud;
        Ok(())
    }

    pub fn write_console(&mut self, data: &[u8]) -> Result<usize, UsbRoleError> {
        if !self.console_active || self.role != UsbRole::Device {
            return Err(UsbRoleError::ConsoleNotReady);
        }
        self.bytes_transmitted += data.len();
        Ok(data.len())
    }

    pub fn current_role(&self) -> UsbRole {
        self.role
    }

    pub fn baud_rate(&self) -> u32 {
        self.baud_rate
    }

    pub fn bytes_transmitted(&self) -> usize {
        self.bytes_transmitted
    }
}
