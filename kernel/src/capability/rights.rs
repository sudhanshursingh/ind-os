//! Capability Rights and Attenuation Logic for IndK
//! Compliant with Bible §8.8 / §10 and Machine Invariant INV-CAP-001 (Monotonic Attenuation).

/// Capability access rights bitflags
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CapRights(pub u32);

impl CapRights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const GRANT: Self = Self(1 << 2); // Authority to delegate/mint to another capability
    pub const CALL: Self = Self(1 << 3);  // Authority to invoke synchronous IPC endpoint
    pub const ALL: Self = Self(Self::READ.0 | Self::WRITE.0 | Self::GRANT.0 | Self::CALL.0);

    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(&self) -> u32 {
        self.0
    }

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersection(&self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Checks monotonic attenuation: `target` rights must be a strict subset of `self` (INV-CAP-001).
    /// Authority can shrink or remain equal along delegation chains, but can NEVER expand.
    pub const fn can_attenuate_to(&self, target: Self) -> bool {
        (target.0 & !self.0) == 0
    }
}
