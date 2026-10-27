//! Kernel Core for Formal Verification of Capability Revocation Soundness (SP-117)
//!
//! Constitutional Invariants:
//! - REQ-SEC-106: Formal verification of capability-core invariants
//! - INV-FORMAL-002: Revocation Soundness & Cascade Completeness
//! - Rule N-01: Zero external crates

pub const KERNEL_CAP_REVOKE_ADMIN: u32 = 0x8000_00B1;
pub const KERNEL_MAX_REVOCATION_NODES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelRevocationError {
    PermissionDenied,
    CapabilityNotFound,
    AlreadyRevoked,
    PoolExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelRevocableNode {
    pub cap_id: u32,
    pub parent_id: u32,
    pub is_revoked: bool,
    pub rights_mask: u32,
    pub active: bool,
}

impl KernelRevocableNode {
    pub const fn empty() -> Self {
        Self {
            cap_id: 0,
            parent_id: 0,
            is_revoked: false,
            rights_mask: 0,
            active: false,
        }
    }
}

pub struct KernelFormalRevocationHarness {
    nodes: [KernelRevocableNode; KERNEL_MAX_REVOCATION_NODES],
    node_count: usize,
    total_revocations: usize,
}

impl KernelFormalRevocationHarness {
    pub const fn new() -> Self {
        Self {
            nodes: [KernelRevocableNode::empty(); KERNEL_MAX_REVOCATION_NODES],
            node_count: 0,
            total_revocations: 0,
        }
    }

    pub fn insert_node(
        &mut self,
        cap_id: u32,
        parent_id: u32,
        rights: u32,
        caller_cap: u32,
    ) -> Result<(), KernelRevocationError> {
        if (caller_cap & KERNEL_CAP_REVOKE_ADMIN) != KERNEL_CAP_REVOKE_ADMIN {
            return Err(KernelRevocationError::PermissionDenied);
        }

        if self.node_count >= KERNEL_MAX_REVOCATION_NODES {
            return Err(KernelRevocationError::PoolExhausted);
        }

        self.nodes[self.node_count] = KernelRevocableNode {
            cap_id,
            parent_id,
            is_revoked: false,
            rights_mask: rights,
            active: true,
        };
        self.node_count += 1;
        Ok(())
    }

    /// Revokes node and all descendants atomically
    pub fn revoke_cascade(
        &mut self,
        target_id: u32,
        caller_cap: u32,
    ) -> Result<usize, KernelRevocationError> {
        if (caller_cap & KERNEL_CAP_REVOKE_ADMIN) != KERNEL_CAP_REVOKE_ADMIN {
            return Err(KernelRevocationError::PermissionDenied);
        }

        let target_idx = self.nodes.iter().position(|n| n.active && n.cap_id == target_id)
            .ok_or(KernelRevocationError::CapabilityNotFound)?;

        if self.nodes[target_idx].is_revoked {
            return Err(KernelRevocationError::AlreadyRevoked);
        }

        self.nodes[target_idx].is_revoked = true;
        let mut count = 1;

        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..self.node_count {
                if self.nodes[i].active && !self.nodes[i].is_revoked {
                    let pid = self.nodes[i].parent_id;
                    if let Some(p_idx) = self.nodes.iter().position(|n| n.active && n.cap_id == pid) {
                        if self.nodes[p_idx].is_revoked {
                            self.nodes[i].is_revoked = true;
                            count += 1;
                            changed = true;
                        }
                    }
                }
            }
        }

        self.total_revocations += count;
        Ok(count)
    }

    pub fn is_active(&self, cap_id: u32) -> bool {
        self.nodes.iter().any(|n| n.active && n.cap_id == cap_id && !n.is_revoked)
    }

    pub fn total_revocations(&self) -> usize {
        self.total_revocations
    }
}
