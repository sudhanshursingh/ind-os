//! Qualcomm UFS 2.2 Controller Driver (SP-74, REQ-HW-105, REQ-KRN-106)
//!
//! Architectural Invariants:
//! - INV-UFS-001: UFSHCI hardware MMIO mapping (0x01d8_4000)
//! - INV-UFS-002: UTP transfer descriptor capability gating
//! - INV-UFS-003: LUN block I/O operations

pub const HANOIP_UFS_BASE: u64 = 0x01d8_4000;
pub const CAP_STORAGE_ADMIN: u32 = 0x0001_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfsDriverError {
    InvalidMmioBase,
    PermissionDenied,
    TransferFailed,
    TableFull,
}

pub struct UfsControllerHarness {
    mmio_mapped: bool,
    active_transfers: u32,
    lun_ready: bool,
}

impl UfsControllerHarness {
    pub const fn new() -> Self {
        Self {
            mmio_mapped: false,
            active_transfers: 0,
            lun_ready: false,
        }
    }

    pub fn map_mmio(&mut self, base: u64) -> Result<(), UfsDriverError> {
        if base != HANOIP_UFS_BASE {
            return Err(UfsDriverError::InvalidMmioBase);
        }
        self.mmio_mapped = true;
        Ok(())
    }

    pub fn submit_utp_transfer(&mut self, cap_mask: u32, lba: u64, blocks: u32) -> Result<(), UfsDriverError> {
        if (cap_mask & CAP_STORAGE_ADMIN) == 0 {
            return Err(UfsDriverError::PermissionDenied);
        }
        if blocks == 0 || lba > 0x1_0000_0000 {
            return Err(UfsDriverError::TransferFailed);
        }
        self.active_transfers += 1;
        self.lun_ready = true;
        Ok(())
    }

    pub fn mmio_mapped(&self) -> bool {
        self.mmio_mapped
    }

    pub fn active_transfers(&self) -> u32 {
        self.active_transfers
    }

    pub fn lun_ready(&self) -> bool {
        self.lun_ready
    }
}
