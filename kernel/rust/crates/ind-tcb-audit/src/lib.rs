//! # Ind OS TCB Audit & Transparency Log
//!
//! Provides hash-chained tamper-evident logging for capability derivations and gateway actions.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    pub sequence_number: u64,
    pub timestamp_ns: u64,
    pub previous_hash: [u8; 32],
    pub payload_digest: [u8; 32],
}

impl AuditEntry {
    pub fn compute_entry_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.sequence_number.to_le_bytes());
        hasher.update(&self.timestamp_ns.to_le_bytes());
        hasher.update(&self.previous_hash);
        hasher.update(&self.payload_digest);
        let result = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&result);
        out
    }
}
