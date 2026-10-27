//! Kernel Boot-Chain Proof & Non-Stock Kernel Validation (SP-72, REQ-HW-103, PEN-005)
//!
//! Architectural Invariants:
//! - INV-BOOT-001: Kernel image packaging & header verification
//! - INV-BOOT-002: AVB 2.0 unlock status & signature policy
//! - INV-BOOT-003: Deterministic boot-fallback & anti-bricking

pub const KERNEL_BOOT_MAGIC: &[u8; 8] = b"ANDROID!";
pub const KERNEL_QUALCOMM_PAGE_SIZE: u32 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelBootChainError {
    InvalidMagic,
    PageSizeMismatch,
    AvbLockedRefusal,
    FallbackTriggered,
}

pub struct KernelBootChainHarness {
    header_verified: bool,
    avb_unlocked: bool,
    fallback_tested: bool,
}

impl KernelBootChainHarness {
    pub const fn new() -> Self {
        Self {
            header_verified: false,
            avb_unlocked: false,
            fallback_tested: false,
        }
    }

    pub fn verify_image_header(&mut self, magic: &[u8], page_size: u32) -> Result<(), KernelBootChainError> {
        if magic.len() != KERNEL_BOOT_MAGIC.len() || magic != KERNEL_BOOT_MAGIC {
            return Err(KernelBootChainError::InvalidMagic);
        }
        if page_size != KERNEL_QUALCOMM_PAGE_SIZE {
            return Err(KernelBootChainError::PageSizeMismatch);
        }
        self.header_verified = true;
        Ok(())
    }

    pub fn check_avb_unlock_state(&mut self, is_unlocked: bool) -> Result<(), KernelBootChainError> {
        if !is_unlocked {
            return Err(KernelBootChainError::AvbLockedRefusal);
        }
        self.avb_unlocked = true;
        Ok(())
    }

    pub fn simulate_boot_failure_fallback(&mut self, recovery_target_slot: &'static str) -> Result<(), KernelBootChainError> {
        if recovery_target_slot.is_empty() {
            return Err(KernelBootChainError::FallbackTriggered);
        }
        self.fallback_tested = true;
        Ok(())
    }

    pub fn header_verified(&self) -> bool {
        self.header_verified
    }

    pub fn avb_unlocked(&self) -> bool {
        self.avb_unlocked
    }

    pub fn fallback_tested(&self) -> bool {
        self.fallback_tested
    }
}
