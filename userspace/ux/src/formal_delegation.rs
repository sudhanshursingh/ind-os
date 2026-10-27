//! Sprint SP-116: Formal Verification of Capability Delegation Monotonicity (Userspace Model)
//!
//! Constitutional Invariants:
//! - REQ-SEC-106: Formal verification of capability-core invariants
//! - INV-FORMAL-001: Monotonic Attenuation (R_child subset_of R_parent)
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in userspace

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormalCapType {
    MemoryMapping,
    IpcChannel,
    DeviceIo,
    StorageAccess,
    ProcessControl,
    AiSubsystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormalDelegationError {
    CapabilityAmplificationViolation,
    TypeMismatch,
    ExceededMaxDepth,
    InvalidParentState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormalCapability {
    pub cap_id: u32,
    pub cap_type: FormalCapType,
    pub rights_mask: u32,
    pub generation: u32,
    pub parent_id: Option<u32>,
    pub depth: u32,
}

impl FormalCapability {
    pub const fn new_root(cap_id: u32, cap_type: FormalCapType, rights_mask: u32) -> Self {
        Self {
            cap_id,
            cap_type,
            rights_mask,
            generation: 1,
            parent_id: None,
            depth: 0,
        }
    }

    /// Attenuates rights to derive a child capability.
    /// Strictly verifies INV-FORMAL-001 (Monotonic Attenuation).
    pub fn delegate_child(
        &self,
        child_id: u32,
        requested_rights: u32,
    ) -> Result<FormalCapability, FormalDelegationError> {
        if self.depth >= 16 {
            return Err(FormalDelegationError::ExceededMaxDepth);
        }

        // INV-FORMAL-001: requested_rights must be a strict subset of self.rights_mask
        // (requested_rights & !self.rights_mask) must be 0
        if (requested_rights & !self.rights_mask) != 0 {
            return Err(FormalDelegationError::CapabilityAmplificationViolation);
        }

        Ok(FormalCapability {
            cap_id: child_id,
            cap_type: self.cap_type,
            rights_mask: requested_rights,
            generation: self.generation + 1,
            parent_id: Some(self.cap_id),
            depth: self.depth + 1,
        })
    }

    /// Checks if a given permission right is held
    pub fn has_right(&self, right_bit: u32) -> bool {
        (self.rights_mask & right_bit) == right_bit
    }
}

/// Formal model validator running inductive step checks
pub struct FormalDelegationValidator {
    checked_transitions: usize,
    violations_detected: usize,
}

impl FormalDelegationValidator {
    pub const fn new() -> Self {
        Self {
            checked_transitions: 0,
            violations_detected: 0,
        }
    }

    pub fn verify_inductive_step(
        &mut self,
        parent: &FormalCapability,
        child_rights: u32,
    ) -> bool {
        self.checked_transitions += 1;
        match parent.delegate_child(parent.cap_id + 1, child_rights) {
            Ok(child) => {
                // Confirm subset property
                let is_monotonic = (child.rights_mask & !parent.rights_mask) == 0;
                if !is_monotonic {
                    self.violations_detected += 1;
                    return false;
                }
                true
            }
            Err(FormalDelegationError::CapabilityAmplificationViolation) => {
                // Confirm that child_rights had bits outside parent
                let had_extra_bits = (child_rights & !parent.rights_mask) != 0;
                if !had_extra_bits {
                    self.violations_detected += 1;
                    return false;
                }
                true
            }
            Err(_) => true,
        }
    }

    pub fn checked_count(&self) -> usize {
        self.checked_transitions
    }

    pub fn violation_count(&self) -> usize {
        self.violations_detected
    }
}
