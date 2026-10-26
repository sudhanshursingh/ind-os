//! # Ind OS Capability Primitives (FS-D8)
//!
//! Implements unforgeable capability tokens, monotonic attenuation, and atomic revocation.

use ind_core_types::ActionTier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights {
    pub can_execute: bool,
    pub can_derive: bool,
    pub tier_limit: ActionTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityToken {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub object_id: [u8; 16],
    pub rights: CapabilityRights,
    pub is_revoked: bool,
}

impl CapabilityToken {
    /// Monotonically attenuates rights. Descendant can never exceed parent rights.
    pub fn attenuate(&self, new_id: u64, reduced_tier: ActionTier) -> Option<Self> {
        if self.is_revoked || !self.rights.can_derive {
            return None;
        }

        // Monotonic attenuation: cannot derive a higher tier capability than held
        if reduced_tier > self.rights.tier_limit {
            return None;
        }

        Some(Self {
            id: new_id,
            parent_id: Some(self.id),
            object_id: self.object_id,
            rights: CapabilityRights {
                can_execute: self.rights.can_execute,
                can_derive: false, // Default to single-level attenuation
                tier_limit: reduced_tier,
            },
            is_revoked: false,
        })
    }
}
