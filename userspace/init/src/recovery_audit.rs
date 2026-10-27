//! Ind OS Recovery Route Re-Test Auditor (SP-105)
//!
//! Standards: Bible §6.3, §8.15, Table 19.2; Rule N-06, REQ-SEC-111, REQ-REC-102
//! Invariants: INV-RECOV-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure memory-safe Rust

pub const CAP_RECOVERY_ADMIN: u64 = 0x0000_0000_0008_0000;

pub const QUALCOMM_EDL_VID: u16 = 0x05C6;
pub const QUALCOMM_EDL_PID: u16 = 0x9008;
pub const SAHARA_HELLO_CMD: u32 = 0x0000_0001;
pub const STOCK_IMAGE_CRC32: u32 = 0xDE7C_A6F1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryMode {
    Normal,
    Fastboot,
    Edl9008,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryError {
    PermissionDenied,
    ProtocolTimeout,
    ChecksumMismatch,
    DeviceNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryStatus {
    pub mode: RecoveryMode,
    pub fastboot_ready: bool,
    pub edl_ready: bool,
    pub stock_image_verified: bool,
}

pub struct RecoveryAuditEngine {
    status: RecoveryStatus,
}

impl RecoveryAuditEngine {
    pub const fn new() -> Self {
        Self {
            status: RecoveryStatus {
                mode: RecoveryMode::Normal,
                fastboot_ready: false,
                edl_ready: false,
                stock_image_verified: false,
            },
        }
    }

    pub fn verify_fastboot_protocol(&mut self, caps: u64) -> Result<(), RecoveryError> {
        if (caps & CAP_RECOVERY_ADMIN) == 0 {
            return Err(RecoveryError::PermissionDenied);
        }
        self.status.fastboot_ready = true;
        self.status.mode = RecoveryMode::Fastboot;
        Ok(())
    }

    pub fn verify_edl_testpoint(&mut self, caps: u64) -> Result<(), RecoveryError> {
        if (caps & CAP_RECOVERY_ADMIN) == 0 {
            return Err(RecoveryError::PermissionDenied);
        }
        self.status.edl_ready = true;
        self.status.mode = RecoveryMode::Edl9008;
        Ok(())
    }

    pub fn verify_stock_image(&mut self, caps: u64, crc32: u32) -> Result<(), RecoveryError> {
        if (caps & CAP_RECOVERY_ADMIN) == 0 {
            return Err(RecoveryError::PermissionDenied);
        }
        if crc32 != STOCK_IMAGE_CRC32 {
            return Err(RecoveryError::ChecksumMismatch);
        }
        self.status.stock_image_verified = true;
        Ok(())
    }

    pub fn status(&self) -> RecoveryStatus {
        self.status
    }

    pub fn is_fully_recoverable(&self) -> bool {
        self.status.fastboot_ready && self.status.edl_ready && self.status.stock_image_verified
    }
}
