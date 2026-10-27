//! Userspace DevMgr Qualcomm USB 3.0 / DWC3 Controller Driver (SP-84, REQ-HW-108, REQ-USB-101)
//!
//! Architectural Invariants:
//! - INV-USB-001: DWC3 Controller MMIO base address mapping (0x04e0_0000)
//! - INV-USB-002: QUSB2 PHY PLL lock and reset sequencing (0x088e_2000)
//! - INV-USB-003: Control Endpoint 0 setup packet processing and descriptor handshake

pub const HANOIP_USB_DWC3_BASE: u64 = 0x04e0_0000;
pub const HANOIP_QUSB2_PHY_BASE: u64 = 0x088e_2000;
pub const USB_REQ_GET_DESCRIPTOR: u8 = 0x06;
pub const USB_REQ_SET_ADDRESS: u8 = 0x05;
pub const USB_DESC_TYPE_DEVICE: u8 = 0x01;
pub const CAP_USB_ADMIN: u32 = 0x0040_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbSpeed {
    Unknown,
    HighSpeed480,
    SuperSpeed5G,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbEndpointState {
    Disabled,
    Stalled,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    InvalidMmioBase,
    PhyNotLocked,
    PermissionDenied,
    InvalidSetupPacket,
    EndpointStalled,
}

pub struct UsbDwc3Harness {
    controller_mapped: bool,
    phy_locked: bool,
    ep0_state: UsbEndpointState,
    device_address: u8,
    speed: UsbSpeed,
}

impl UsbDwc3Harness {
    pub const fn new() -> Self {
        Self {
            controller_mapped: false,
            phy_locked: false,
            ep0_state: UsbEndpointState::Disabled,
            device_address: 0,
            speed: UsbSpeed::Unknown,
        }
    }

    pub fn map_mmio(&mut self, controller_base: u64, phy_base: u64) -> Result<(), UsbError> {
        if controller_base != HANOIP_USB_DWC3_BASE || phy_base != HANOIP_QUSB2_PHY_BASE {
            return Err(UsbError::InvalidMmioBase);
        }
        self.controller_mapped = true;
        Ok(())
    }

    pub fn lock_qusb2_phy(&mut self) -> Result<(), UsbError> {
        if !self.controller_mapped {
            return Err(UsbError::InvalidMmioBase);
        }
        self.phy_locked = true;
        self.speed = UsbSpeed::HighSpeed480;
        self.ep0_state = UsbEndpointState::Ready;
        Ok(())
    }

    pub fn handle_setup_packet(
        &mut self,
        _req_type: u8,
        request: u8,
        value: u16,
        _index: u16,
        length: u16,
    ) -> Result<u16, UsbError> {
        if !self.phy_locked {
            return Err(UsbError::PhyNotLocked);
        }
        if self.ep0_state != UsbEndpointState::Ready {
            return Err(UsbError::EndpointStalled);
        }

        match request {
            USB_REQ_GET_DESCRIPTOR => {
                let desc_type = (value >> 8) as u8;
                if desc_type == USB_DESC_TYPE_DEVICE {
                    // Standard device descriptor length is 18 bytes
                    let returned = if length < 18 { length } else { 18 };
                    Ok(returned)
                } else {
                    Err(UsbError::InvalidSetupPacket)
                }
            }
            USB_REQ_SET_ADDRESS => {
                let addr = (value & 0x7F) as u8;
                self.device_address = addr;
                Ok(0)
            }
            _ => Err(UsbError::InvalidSetupPacket),
        }
    }

    pub fn is_controller_mapped(&self) -> bool {
        self.controller_mapped
    }

    pub fn is_phy_locked(&self) -> bool {
        self.phy_locked
    }

    pub fn ep0_state(&self) -> UsbEndpointState {
        self.ep0_state
    }

    pub fn device_address(&self) -> u8 {
        self.device_address
    }

    pub fn speed(&self) -> UsbSpeed {
        self.speed
    }
}
