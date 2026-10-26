//! Consent Record Schema (Consent Broker v1)
//! Compliant with Bible §8.9 (SP-21), §10, §14 and Invariant INV-CON-001.

use crate::capability::rights::CapRights;
use crate::task::ProcessId;

/// Explicit user decision status for a consent request
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ConsentDecision {
    Pending = 0,
    Granted = 1,
    Denied = 2,
    TimedOut = 3,
}

/// Structured, cryptographically auditable consent record
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsentRecord {
    pub consent_id: u64,
    pub requesting_pid: ProcessId,
    pub resource_id: u32,
    pub requested_rights: CapRights,
    pub granted_rights: CapRights,
    pub decision: ConsentDecision,
    pub timestamp: u64,
    pub audit_seq: u64,
}

impl ConsentRecord {
    pub const fn empty() -> Self {
        Self {
            consent_id: 0,
            requesting_pid: 0,
            resource_id: 0,
            requested_rights: CapRights::NONE,
            granted_rights: CapRights::NONE,
            decision: ConsentDecision::Pending,
            timestamp: 0,
            audit_seq: 0,
        }
    }
}
