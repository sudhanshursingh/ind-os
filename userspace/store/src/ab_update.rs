//! Ind OS Hardware A/B Dual-Slot Update Service (SP-102)
//!
//! Standards: Bible §6.3, §8.15, Table 19.2; REQ-SEC-112, REQ-STR-104
//! Invariants: INV-OTA-AB-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure memory-safe Rust

pub const CAP_STORAGE_ADMIN: u64 = 0x0000_0000_0002_0000;
pub const MAX_BOOT_TRIES: u8 = 3;
pub const PAYLOAD_BLOCK_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotId {
    SlotA,
    SlotB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbUpdateError {
    PermissionDenied,
    InvalidSlot,
    IntegrityMismatch,
    SlotNotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotMetadata {
    pub slot: SlotId,
    pub is_active: bool,
    pub is_successful: bool,
    pub tries_remaining: u8,
}

pub struct AbUpdateEngine {
    slot_a: SlotMetadata,
    slot_b: SlotMetadata,
    staged_blocks: usize,
    rollback_armed: bool,
}

impl AbUpdateEngine {
    pub const fn new() -> Self {
        Self {
            slot_a: SlotMetadata {
                slot: SlotId::SlotA,
                is_active: true,
                is_successful: true,
                tries_remaining: 0,
            },
            slot_b: SlotMetadata {
                slot: SlotId::SlotB,
                is_active: false,
                is_successful: false,
                tries_remaining: 0,
            },
            staged_blocks: 0,
            rollback_armed: false,
        }
    }

    pub fn get_slot_info(&self, slot: SlotId) -> SlotMetadata {
        match slot {
            SlotId::SlotA => self.slot_a,
            SlotId::SlotB => self.slot_b,
        }
    }

    pub fn active_slot(&self) -> SlotId {
        if self.slot_a.is_active {
            SlotId::SlotA
        } else {
            SlotId::SlotB
        }
    }

    pub fn stage_payload(
        &mut self,
        caps: u64,
        target_slot: SlotId,
        payload_checksum: u32,
        expected_checksum: u32,
    ) -> Result<usize, AbUpdateError> {
        if (caps & CAP_STORAGE_ADMIN) == 0 {
            return Err(AbUpdateError::PermissionDenied);
        }
        if target_slot == self.active_slot() {
            return Err(AbUpdateError::InvalidSlot);
        }
        if payload_checksum != expected_checksum {
            return Err(AbUpdateError::IntegrityMismatch);
        }

        self.staged_blocks = 2048; // 8MB payload
        Ok(self.staged_blocks)
    }

    pub fn activate_slot(&mut self, caps: u64, target_slot: SlotId) -> Result<(), AbUpdateError> {
        if (caps & CAP_STORAGE_ADMIN) == 0 {
            return Err(AbUpdateError::PermissionDenied);
        }
        if self.staged_blocks == 0 {
            return Err(AbUpdateError::SlotNotReady);
        }

        match target_slot {
            SlotId::SlotA => {
                self.slot_a.is_active = true;
                self.slot_a.is_successful = false;
                self.slot_a.tries_remaining = MAX_BOOT_TRIES;
                self.slot_b.is_active = false;
            }
            SlotId::SlotB => {
                self.slot_b.is_active = true;
                self.slot_b.is_successful = false;
                self.slot_b.tries_remaining = MAX_BOOT_TRIES;
                self.slot_a.is_active = false;
            }
        }
        self.rollback_armed = true;
        Ok(())
    }

    pub fn mark_boot_successful(&mut self, caps: u64) -> Result<(), AbUpdateError> {
        if (caps & CAP_STORAGE_ADMIN) == 0 {
            return Err(AbUpdateError::PermissionDenied);
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
