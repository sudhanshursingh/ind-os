//! # Ind OS Capability System (Stage A / Stage B)
//!
//! Formal implementation mirroring the TLA+ specifications (`IndCapability.tla`, `IndCapabilityHierarchy.tla`):
//! - Monotonic attenuation (rights can never grow).
//! - Atomic revocation (revoking a parent invalidates all transitive descendants).
//! - Red Derivation Theorem (no RED capability without verified biometric constraint).

use ind_core_types::{ActionTier, DenialCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityConstraint {
    UserVoiceConfirmed,
    UserPresenceBiometric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights {
    pub can_read: bool,
    pub can_write: bool,
    pub can_execute: bool,
    pub can_derive: bool,
    pub tier_limit: ActionTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityToken {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub object_uri: String,
    pub rights: CapabilityRights,
    pub constraints: Vec<CapabilityConstraint>,
    pub is_revoked: bool,
}

impl CapabilityToken {
    /// Evaluates if capability can be exercised for a target tier
    pub fn can_exercise(&self, requested_tier: ActionTier) -> Result<(), DenialCode> {
        if self.is_revoked {
            return Err(DenialCode::UnauthorizedCapability);
        }

        if !self.rights.can_execute {
            return Err(DenialCode::UnauthorizedCapability);
        }

        if requested_tier > self.rights.tier_limit {
            return Err(DenialCode::UnauthorizedCapability);
        }

        if requested_tier == ActionTier::Amber {
            let has_voice = self.constraints.contains(&CapabilityConstraint::UserVoiceConfirmed)
                || self.constraints.contains(&CapabilityConstraint::UserPresenceBiometric);
            if !has_voice {
                return Err(DenialCode::MissingVoiceConfirmation);
            }
        }

        if requested_tier == ActionTier::Red {
            let has_bio = self.constraints.contains(&CapabilityConstraint::UserPresenceBiometric);
            if !has_bio {
                return Err(DenialCode::MissingBiometricProof);
            }
        }

        Ok(())
    }

    /// Monotonically attenuates rights into a child capability
    pub fn attenuate(
        &self,
        new_id: u64,
        reduced_tier: ActionTier,
        added_constraints: &[CapabilityConstraint],
    ) -> Option<Self> {
        if self.is_revoked || !self.rights.can_derive {
            return None;
        }

        // Monotonic attenuation law: cannot elevate tier
        if reduced_tier > self.rights.tier_limit {
            return None;
        }

        // RED Derivation Theorem: child cannot be RED unless parent or child holds biometric proof
        let mut combined_constraints = self.constraints.clone();
        for &c in added_constraints {
            if !combined_constraints.contains(&c) {
                combined_constraints.push(c);
            }
        }

        if reduced_tier == ActionTier::Red
            && !combined_constraints.contains(&CapabilityConstraint::UserPresenceBiometric)
        {
            return None;
        }

        Some(Self {
            id: new_id,
            parent_id: Some(self.id),
            object_uri: self.object_uri.clone(),
            rights: CapabilityRights {
                can_read: self.rights.can_read,
                can_write: self.rights.can_write,
                can_execute: self.rights.can_execute,
                can_derive: false, // Monotonic attenuation stops nested derivation by default
                tier_limit: reduced_tier,
            },
            constraints: combined_constraints,
            is_revoked: false,
        })
    }
}

/// In-memory Capability Table with atomic cascade revocation
#[derive(Debug, Default)]
pub struct CapabilityTable {
    tokens: Vec<CapabilityToken>,
}

impl CapabilityTable {
    pub fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn insert(&mut self, token: CapabilityToken) {
        self.tokens.push(token);
    }

    pub fn get(&self, id: u64) -> Option<&CapabilityToken> {
        self.tokens.iter().find(|t| t.id == id)
    }

    /// Atomically revokes a capability and all of its descendants
    pub fn revoke_cascade(&mut self, target_id: u64) {
        let mut to_revoke = vec![target_id];
        let mut i = 0;
        while i < to_revoke.len() {
            let curr = to_revoke[i];
            for t in &self.tokens {
                if t.parent_id == Some(curr) && !to_revoke.contains(&t.id) {
                    to_revoke.push(t.id);
                }
            }
            i += 1;
        }

        for t in &mut self.tokens {
            if to_revoke.contains(&t.id) {
                t.is_revoked = true;
            }
        }
    }
}
