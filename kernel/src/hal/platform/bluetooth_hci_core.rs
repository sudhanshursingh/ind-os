//! Kernel Qualcomm Bluetooth HCI Driver (SP-88, REQ-HW-109, REQ-NET-102)
//!
//! Architectural Invariants:
//! - INV-BT-001: GENI UART HCI transport MMIO base mapping (0x0884_0000)
//! - INV-BT-002: Firmware NVM patch download and chip reset
//! - INV-BT-003: HCI command loop and BLE advertisement scanner

pub const KERNEL_HANOIP_BT_UART_BASE: u64 = 0x0884_0000;
pub const KERNEL_HCI_CMD_RESET: u16 = 0x0C03;
pub const KERNEL_HCI_CMD_LE_SET_SCAN_ENABLE: u16 = 0x200C;
pub const KERNEL_CAP_BT_ADMIN: u32 = 0x0400_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelBtState {
    PoweredOff,
    ResetPending,
    Ready,
    Scanning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelBtError {
    InvalidMmioBase,
    FirmwareNotStaged,
    PermissionDenied,
    InvalidHciCommand,
    BufferOverflow,
}

pub struct KernelBluetoothHciHarness {
    mmio_mapped: bool,
    fw_loaded: bool,
    state: KernelBtState,
    commands_processed: usize,
}

impl KernelBluetoothHciHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            fw_loaded: false,
            state: KernelBtState::PoweredOff,
            commands_processed: 0,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), KernelBtError> {
        if base != KERNEL_HANOIP_BT_UART_BASE {
            return Err(KernelBtError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn load_firmware_patch(&mut self, fw_staged: bool) -> Result<(), KernelBtError> {
        if !self.mmio_mapped {
            return Err(KernelBtError::InvalidMmioBase);
        }
        if !fw_staged {
            return Err(KernelBtError::FirmwareNotStaged);
        }
        self.fw_loaded = true;
        self.state = KernelBtState::ResetPending;
        Ok(())
    }

    pub fn send_hci_command(
        &mut self,
        cap_mask: u32,
        opcode: u16,
        _params: &[u8],
    ) -> Result<u8, KernelBtError> {
        if (cap_mask & KERNEL_CAP_BT_ADMIN) == 0 {
            return Err(KernelBtError::PermissionDenied);
        }
        if !self.fw_loaded {
            return Err(KernelBtError::FirmwareNotStaged);
        }

        match opcode {
            KERNEL_HCI_CMD_RESET => {
                self.state = KernelBtState::Ready;
                self.commands_processed += 1;
                Ok(0x00)
            }
            KERNEL_HCI_CMD_LE_SET_SCAN_ENABLE => {
                if self.state != KernelBtState::Ready && self.state != KernelBtState::Scanning {
                    return Err(KernelBtError::InvalidHciCommand);
                }
                self.state = KernelBtState::Scanning;
                self.commands_processed += 1;
                Ok(0x00)
            }
            _ => Err(KernelBtError::InvalidHciCommand),
        }
    }

    pub fn state(&self) -> KernelBtState {
        self.state
    }

    pub fn is_mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn is_fw_loaded(&self) -> bool {
        self.fw_loaded
    }

    pub fn commands_processed(&self) -> usize {
        self.commands_processed
    }
}
