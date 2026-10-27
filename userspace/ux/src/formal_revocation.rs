//! Sprint SP-117: Formal Verification of Capability Revocation Soundness (Userspace Model)
//!
//! Constitutional Invariants:
//! - REQ-SEC-106: Formal verification of capability-core invariants
//! - INV-FORMAL-002: Revocation Soundness & Cascade Completeness
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in userspace

pub const MAX_TREE_CAPACITIES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationError {
    CapabilityNotFound,
    AlreadyRevoked,
    PermissionDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocableCapNode {
    pub cap_id: u32,
    pub parent_id: u32,
    pub is_revoked: bool,
    pub rights_mask: u32,
}

impl RevocableCapNode {
    pub const fn empty() -> Self {
        Self {
            cap_id: 0,
            parent_id: 0,
            is_revoked: false,
            rights_mask: 0,
        }
    }
}

pub struct FormalRevocationModel {
    nodes: [RevocableCapNode; MAX_TREE_CAPACITIES],
    count: usize,
}

impl FormalRevocationModel {
    pub const fn new() -> Self {
        Self {
            nodes: [RevocableCapNode::empty(); MAX_TREE_CAPACITIES],
            count: 0,
        }
    }

    pub fn insert_node(&mut self, cap_id: u32, parent_id: u32, rights: u32) -> Result<(), RevocationError> {
        if self.count >= MAX_TREE_CAPACITIES {
            return Err(RevocationError::PermissionDenied);
        }
        self.nodes[self.count] = RevocableCapNode {
            cap_id,
            parent_id,
            is_revoked: false,
            rights_mask: rights,
        };
        self.count += 1;
        Ok(())
    }

    /// Revokes target capability and cascades to all transitive descendants
    pub fn revoke_cascade(&mut self, target_id: u32) -> Result<usize, RevocationError> {
        let target_idx = self.nodes.iter().position(|n| n.cap_id == target_id)
            .ok_or(RevocationError::CapabilityNotFound)?;

        if self.nodes[target_idx].is_revoked {
            return Err(RevocationError::AlreadyRevoked);
        }

        // Mark target
        self.nodes[target_idx].is_revoked = true;
        let mut revoked_count = 1;

        // Cascade mark iterative fixpoint
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..self.count {
                if !self.nodes[i].is_revoked {
                    let parent_id = self.nodes[i].parent_id;
                    // Check if parent is revoked
                    if let Some(p_idx) = self.nodes.iter().position(|n| n.cap_id == parent_id) {
                        if self.nodes[p_idx].is_revoked {
                            self.nodes[i].is_revoked = true;
                            revoked_count += 1;
                            changed = true;
                        }
                    }
                }
            }
        }

        Ok(revoked_count)
    }

    pub fn is_active(&self, cap_id: u32) -> bool {
        self.nodes.iter().any(|n| n.cap_id == cap_id && !n.is_revoked)
    }
}
