//! Kernel Core for Formal Verification of Capability Delegation Monotonicity (SP-116)
//!
//! Constitutional Invariants:
//! - REQ-SEC-106: Formal verification of capability-core invariants
//! - INV-FORMAL-001: Monotonic Attenuation
//! - Rule N-01: Zero external crates

pub const KERNEL_CAP_FORMAL_VERIFY: u32 = 0x8000_00B0;
pub const KERNEL_MAX_FORMAL_NODES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelFormalCapError {
    PermissionDenied,
    CapabilityAmplificationViolation,
    NodePoolExhausted,
    InvalidNode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelFormalCapNode {
    pub node_id: u32,
    pub rights_mask: u32,
    pub generation: u32,
    pub parent_id: u32,
    pub depth: u32,
    pub active: bool,
}

impl KernelFormalCapNode {
    pub const fn empty() -> Self {
        Self {
            node_id: 0,
            rights_mask: 0,
            generation: 0,
            parent_id: 0,
            depth: 0,
            active: false,
        }
    }
}

pub struct KernelFormalDelegationHarness {
    nodes: [KernelFormalCapNode; KERNEL_MAX_FORMAL_NODES],
    node_count: usize,
    verified_transitions: usize,
}

impl KernelFormalDelegationHarness {
    pub const fn new() -> Self {
        Self {
            nodes: [KernelFormalCapNode::empty(); KERNEL_MAX_FORMAL_NODES],
            node_count: 0,
            verified_transitions: 0,
        }
    }

    pub fn init_root(&mut self, root_rights: u32, caller_cap: u32) -> Result<u32, KernelFormalCapError> {
        if (caller_cap & KERNEL_CAP_FORMAL_VERIFY) != KERNEL_CAP_FORMAL_VERIFY {
            return Err(KernelFormalCapError::PermissionDenied);
        }

        self.nodes[0] = KernelFormalCapNode {
            node_id: 1,
            rights_mask: root_rights,
            generation: 1,
            parent_id: 0,
            depth: 0,
            active: true,
        };
        self.node_count = 1;
        Ok(1)
    }

    /// Derives a child capability with strict monotonic attenuation checking
    pub fn delegate_child(
        &mut self,
        parent_id: u32,
        requested_rights: u32,
        caller_cap: u32,
    ) -> Result<u32, KernelFormalCapError> {
        if (caller_cap & KERNEL_CAP_FORMAL_VERIFY) != KERNEL_CAP_FORMAL_VERIFY {
            return Err(KernelFormalCapError::PermissionDenied);
        }

        if self.node_count >= KERNEL_MAX_FORMAL_NODES {
            return Err(KernelFormalCapError::NodePoolExhausted);
        }

        let parent_idx = self.nodes.iter().position(|n| n.active && n.node_id == parent_id)
            .ok_or(KernelFormalCapError::InvalidNode)?;
        
        let parent = self.nodes[parent_idx];

        // INV-FORMAL-001: Monotonic Attenuation
        if (requested_rights & !parent.rights_mask) != 0 {
            return Err(KernelFormalCapError::CapabilityAmplificationViolation);
        }

        let new_id = (self.node_count + 1) as u32;
        self.nodes[self.node_count] = KernelFormalCapNode {
            node_id: new_id,
            rights_mask: requested_rights,
            generation: parent.generation + 1,
            parent_id,
            depth: parent.depth + 1,
            active: true,
        };
        self.node_count += 1;
        self.verified_transitions += 1;

        Ok(new_id)
    }

    pub fn node_count(&self) -> usize {
        self.node_count
    }

    pub fn verified_transitions(&self) -> usize {
        self.verified_transitions
    }
}
