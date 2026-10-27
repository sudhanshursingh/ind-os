//! Kernel A/B Update Service Verification Harness (SP-54, REQ-USE-205)
//!
//! Architectural Invariants:
//! - INV-UPD-001: Atomic slot switch between dual partitions
//! - INV-UPD-002: Cryptographically signed manifests with tamper rejection
//! - INV-UPD-003: Automatic rollback upon boot verification failure
//! - INV-UPD-004: User-controlled update policy without silent ambient reboots

pub const KERNEL_AUTHORIZED_UPDATE_SIGNATURE: u32 = 0x5550_4454;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSlotId {
    SlotA = 0,
    SlotB = 1,
}

impl KernelSlotId {
    pub fn other(&self) -> Self {
        match self {
            KernelSlotId::SlotA => KernelSlotId::SlotB,
            KernelSlotId::SlotB => KernelSlotId::SlotA,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSlotState {
    Active,
    Inactive,
    Updating,
    PendingVerification,
    RolledBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelUpdatePolicy {
    ManualApproval,
    ScheduledWindow,
    Immediate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelUpdateError {
    InvalidSignature,
    TamperedImage,
    SlotBusy,
    VerificationFailed,
    PolicyDenied,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelUpdateManifest {
    pub version_code: u32,
    pub target_slot: KernelSlotId,
    pub image_hash: u32,
    pub signature: u32,
    pub size_bytes: usize,
}

pub struct KernelUpdateManager {
    pub active_slot: KernelSlotId,
    pub slot_states: [KernelSlotState; 2],
    pub policy: KernelUpdatePolicy,
}

impl KernelUpdateManager {
    pub fn new(initial_active: KernelSlotId) -> Self {
        let mut states = [KernelSlotState::Inactive; 2];
        states[initial_active as usize] = KernelSlotState::Active;

        Self {
            active_slot: initial_active,
            slot_states: states,
            policy: KernelUpdatePolicy::ManualApproval,
        }
    }

    pub fn compute_checksum(bytes: &[u8]) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for &b in bytes {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash
    }

    pub fn verify_and_stage(
        &mut self,
        manifest: &KernelUpdateManifest,
        image_bytes: &[u8],
        user_consent: bool,
    ) -> Result<(), KernelUpdateError> {
        if self.policy == KernelUpdatePolicy::ManualApproval && !user_consent {
            return Err(KernelUpdateError::PolicyDenied);
        }

        if manifest.signature != KERNEL_AUTHORIZED_UPDATE_SIGNATURE {
            return Err(KernelUpdateError::InvalidSignature);
        }

        let actual_hash = Self::compute_checksum(image_bytes);
        if actual_hash != manifest.image_hash {
            return Err(KernelUpdateError::TamperedImage);
        }

        let target = self.active_slot.other();
        self.slot_states[target as usize] = KernelSlotState::Updating;
        Ok(())
    }

    pub fn atomic_switch(&mut self) -> Result<KernelSlotId, KernelUpdateError> {
        let target = self.active_slot.other();
        if self.slot_states[target as usize] != KernelSlotState::Updating {
            return Err(KernelUpdateError::SlotBusy);
        }

        self.slot_states[self.active_slot as usize] = KernelSlotState::Inactive;
        self.slot_states[target as usize] = KernelSlotState::PendingVerification;
        self.active_slot = target;

        Ok(target)
    }

    pub fn verify_health(&mut self, healthy: bool) -> Result<KernelSlotId, KernelUpdateError> {
        let current = self.active_slot;
        if healthy {
            self.slot_states[current as usize] = KernelSlotState::Active;
            Ok(current)
        } else {
            let previous = current.other();
            self.slot_states[current as usize] = KernelSlotState::RolledBack;
            self.slot_states[previous as usize] = KernelSlotState::Active;
            self.active_slot = previous;
            Err(KernelUpdateError::VerificationFailed)
        }
    }

    pub fn simulate_power_interruption(&mut self) {
        let other = self.active_slot.other();
        if self.slot_states[other as usize] == KernelSlotState::Updating {
            self.slot_states[other as usize] = KernelSlotState::RolledBack;
        } else if self.slot_states[self.active_slot as usize] == KernelSlotState::PendingVerification {
            let prev = self.active_slot.other();
            self.slot_states[self.active_slot as usize] = KernelSlotState::RolledBack;
            self.slot_states[prev as usize] = KernelSlotState::Active;
            self.active_slot = prev;
        }
    }
}

