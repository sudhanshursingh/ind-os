//! Cryptographic Audit Log Entry & Event Types (Audit v1)
//! Compliant with Bible §8.9 (SP-20), §10, §14 and Invariant INV-AUD-001.

use crate::audit::sha256::Sha256;
use crate::capability::rights::CapRights;
use crate::task::ProcessId;

/// Audit security event classifications
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum AuditEventType {
    Genesis = 0,
    CapabilityGrant = 1,
    CapabilityRevoke = 2,
    CapabilityDelegate = 3,
    PolicyAllow = 4,
    PolicyDeny = 5,
    PolicyConsentRequired = 6,
    PolicyUpdate = 7,
    ProcessTermination = 8,
}

/// Cryptographically chained append-only audit entry (INV-AUD-001)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditEntry {
    pub sequence: u64,
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub subject_pid: ProcessId,
    pub target_id: u32,
    pub rights: CapRights,
    pub prev_hash: [u8; 32],
    pub entry_hash: [u8; 32],
}

impl AuditEntry {
    pub const fn empty() -> Self {
        Self {
            sequence: 0,
            timestamp: 0,
            event_type: AuditEventType::Genesis,
            subject_pid: 0,
            target_id: 0,
            rights: CapRights::NONE,
            prev_hash: [0u8; 32],
            entry_hash: [0u8; 32],
        }
    }

    /// Computes the cryptographic SHA-256 hash of this entry over its contents and previous hash (INV-AUD-001)
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.sequence.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&(self.event_type as u32).to_be_bytes());
        hasher.update(&self.subject_pid.to_be_bytes());
        hasher.update(&self.target_id.to_be_bytes());
        hasher.update(&self.rights.bits().to_be_bytes());
        hasher.update(&self.prev_hash);
        hasher.finalize()
    }
}
