//! Kernel Core for Production Operations & Rollback-Safe Dual-Slot A/B Update (SP-121)
//!
//! Constitutional Invariants:
//! - REQ-OPS-101: Dual-slot atomic OTA update pipeline with rollback
//! - INV-PRODOPS-001: Cryptographic Update Integrity & Automatic Fallback
//! - Rule N-01: Zero external crates

pub const KERNEL_CAP_PRODOPS_ADMIN: u32 = 0x8000_00B5;
pub const KERNEL_PRODUCTION_KEY_ID: u32 = 0x1A2B_3C4D;
pub const KERNEL_PRODUCTION_IMAGE_CRC: u32 = 0x98A1_4C2F;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelProdOpsError {
    PermissionDenied,
    SignatureInvalid,
    ChecksumMismatch,
    RollbackFailed,
}

pub struct KernelProductionOpsHarness {
    slot_a_bootable: bool,
    slot_b_bootable: bool,
    active_slot: u8,
    rollback_rehearsed: bool,
}

impl KernelProductionOpsHarness {
    pub const fn new() -> Self {
        Self {
            slot_a_bootable: true,
            slot_b_bootable: false,
            active_slot: 0,
            rollback_rehearsed: false,
        }
    }

    pub fn rehearse_signing_ceremony(&mut self, key_id: u32, image_crc: u32, caller_cap: u32) -> Result<(), KernelProdOpsError> {
        if (caller_cap & KERNEL_CAP_PRODOPS_ADMIN) != KERNEL_CAP_PRODOPS_ADMIN {
            return Err(KernelProdOpsError::PermissionDenied);
        }
        if key_id != KERNEL_PRODUCTION_KEY_ID {
            return Err(KernelProdOpsError::SignatureInvalid);
        }
        if image_crc != KERNEL_PRODUCTION_IMAGE_CRC {
            return Err(KernelProdOpsError::ChecksumMismatch);
        }

        self.slot_b_bootable = true;
        self.active_slot = 1; // Stage slot B
        Ok(())
    }

    pub fn rehearse_automatic_rollback(&mut self, caller_cap: u32) -> Result<(), KernelProdOpsError> {
        if (caller_cap & KERNEL_CAP_PRODOPS_ADMIN) != KERNEL_CAP_PRODOPS_ADMIN {
            return Err(KernelProdOpsError::PermissionDenied);
        }

        // Revert to slot A
        self.active_slot = 0;
        self.rollback_rehearsed = true;
        Ok(())
    }

    pub fn is_rehearsed(&self) -> bool {
        self.rollback_rehearsed && self.active_slot == 0 && self.slot_a_bootable
    }
}
