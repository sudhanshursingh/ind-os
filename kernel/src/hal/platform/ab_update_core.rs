//! Kernel Hardware A/B Dual-Slot Update Core (SP-102, REQ-SEC-112, REQ-STR-104)
//!
//! A/B slot partition metadata management, payload staging, and rollback armed state machine.

pub const KERNEL_CAP_AB_UPDATE: u32 = 0x0002_0000;
pub const KERNEL_MAX_BOOT_TRIES: u8 = 3;
pub const KERNEL_PAYLOAD_BLOCK_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSlotId {
    SlotA,
    SlotB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAbUpdateError {
    PermissionDenied,
    InvalidSlot,
    IntegrityMismatch,
    SlotNotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSlotMetadata {
    pub slot: KernelSlotId,
    pub is_active: bool,
    pub is_successful: bool,
    pub tries_remaining: u8,
}

pub struct KernelAbUpdateHarness {
    slot_a: KernelSlotMetadata,
    slot_b: KernelSlotMetadata,
    staged_blocks: usize,
    rollback_armed: bool,
}

impl KernelAbUpdateHarness {
    pub const fn new() -> Self {
        Self {
            slot_a: KernelSlotMetadata {
                slot: KernelSlotId::SlotA,
                is_active: true,
                is_successful: true,
                tries_remaining: 0,
            },
            slot_b: KernelSlotMetadata {
                slot: KernelSlotId::SlotB,
                is_active: false,
                is_successful: false,
                tries_remaining: 0,
            },
            staged_blocks: 0,
            rollback_armed: false,
        }
    }

    pub fn get_slot_info(&self, slot: KernelSlotId) -> KernelSlotMetadata {
        match slot {
            KernelSlotId::SlotA => self.slot_a,
            KernelSlotId::SlotB => self.slot_b,
        }
    }

    pub fn active_slot(&self) -> KernelSlotId {
        if self.slot_a.is_active {
            KernelSlotId::SlotA
        } else {
            KernelSlotId::SlotB
        }
    }

    pub fn stage_payload(
        &mut self,
        cap_token: u32,
        target_slot: KernelSlotId,
        payload_checksum: u32,
        expected_checksum: u32,
    ) -> Result<usize, KernelAbUpdateError> {
        if (cap_token & KERNEL_CAP_AB_UPDATE) == 0 {
            return Err(KernelAbUpdateError::PermissionDenied);
        }
        if target_slot == self.active_slot() {
            return Err(KernelAbUpdateError::InvalidSlot);
        }
        if payload_checksum != expected_checksum {
            return Err(KernelAbUpdateError::IntegrityMismatch);
        }

        self.staged_blocks = 2048; // 8MB payload
        Ok(self.staged_blocks)
    }

    pub fn activate_slot(&mut self, cap_token: u32, target_slot: KernelSlotId) -> Result<(), KernelAbUpdateError> {
        if (cap_token & KERNEL_CAP_AB_UPDATE) == 0 {
            return Err(KernelAbUpdateError::PermissionDenied);
        }
        if self.staged_blocks == 0 {
            return Err(KernelAbUpdateError::SlotNotReady);
        }

        match target_slot {
            KernelSlotId::SlotA => {
                self.slot_a.is_active = true;
                self.slot_a.is_successful = false;
                self.slot_a.tries_remaining = KERNEL_MAX_BOOT_TRIES;
                self.slot_b.is_active = false;
            }
            KernelSlotId::SlotB => {
                self.slot_b.is_active = true;
                self.slot_b.is_successful = false;
                self.slot_b.tries_remaining = KERNEL_MAX_BOOT_TRIES;
                self.slot_a.is_active = false;
            }
        }
        self.rollback_armed = true;
        Ok(())
    }

    pub fn mark_boot_successful(&mut self, cap_token: u32) -> Result<(), KernelAbUpdateError> {
        if (cap_token & KERNEL_CAP_AB_UPDATE) == 0 {
            return Err(KernelAbUpdateError::PermissionDenied);
        }
        if self.slot_a.is_active {
            self.slot_a.is_successful = true;
            self.slot_a.tries_remaining = 0;
        } else {
            self.slot_b.is_successful = true;
            self.slot_b.tries_remaining = 0;
        }
        self.rollback_armed = false;
        Ok(())
    }

    pub fn staged_blocks(&self) -> usize {
        self.staged_blocks
    }

    pub fn is_rollback_armed(&self) -> bool {
        self.rollback_armed
    }
}
