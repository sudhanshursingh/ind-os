//! Consent Broker Implementation (Consent Broker v1)
//! Compliant with Bible §8.9 (SP-21), §10, §14 and Invariant INV-CON-001.

use crate::audit::{AuditEventType, AUDIT_LOG};
use crate::capability::rights::CapRights;
use crate::consent::record::{ConsentDecision, ConsentRecord};
use crate::consent::surface::IsolatedSurface;
use crate::task::ProcessId;

/// Maximum number of active / historical consent records stored in bounded static memory
pub const MAX_CONSENT_RECORDS: usize = 32;

/// Trusted Consent Broker state machine and mediation engine
pub struct ConsentBroker {
    surface: IsolatedSurface,
    records: [ConsentRecord; MAX_CONSENT_RECORDS],
    count: usize,
    next_id: u64,
}

impl ConsentBroker {
    pub const fn new() -> Self {
        Self {
            surface: IsolatedSurface::new(),
            records: [ConsentRecord::empty(); MAX_CONSENT_RECORDS],
            count: 0,
            next_id: 1,
        }
    }

    /// Requests user consent for a sensitive resource access, activating the isolated display plane
    pub fn request_consent(
        &mut self,
        requesting_pid: ProcessId,
        resource_id: u32,
        requested_rights: CapRights,
        timestamp: u64,
    ) -> Result<u64, &'static str> {
        if self.count >= MAX_CONSENT_RECORDS {
            return Err("Consent record capacity exceeded");
        }

        // Activate isolated hardware surface and preempt user input
        self.surface.activate()?;

        let consent_id = self.next_id;
        self.next_id += 1;

        // Log PolicyConsentRequired event in cryptographic audit log
        let audit_seq = {
            let mut audit = AUDIT_LOG.lock();
            audit
                .record(
                    AuditEventType::PolicyConsentRequired,
                    requesting_pid,
                    resource_id,
                    requested_rights,
                    timestamp,
                )
                .unwrap_or(0)
        };

        let record = ConsentRecord {
            consent_id,
            requesting_pid,
            resource_id,
            requested_rights,
            granted_rights: CapRights::NONE,
            decision: ConsentDecision::Pending,
            timestamp,
            audit_seq,
        };

        self.records[self.count] = record;
        self.count += 1;

        Ok(consent_id)
    }

    /// Submits the explicit user decision from the trusted UI channel
    pub fn submit_decision(
        &mut self,
        consent_id: u64,
        decision: ConsentDecision,
        granted_rights: CapRights,
        timestamp: u64,
    ) -> Result<ConsentRecord, &'static str> {
        let mut found_idx = None;
        for i in 0..self.count {
            if self.records[i].consent_id == consent_id {
                found_idx = Some(i);
                break;
            }
        }

        let idx = match found_idx {
            Some(i) => i,
            None => return Err("Consent ID not found"),
        };

        if self.records[idx].decision != ConsentDecision::Pending {
            return Err("Consent request is already completed");
        }

        self.records[idx].decision = decision;
        self.records[idx].granted_rights = if decision == ConsentDecision::Granted {
            granted_rights
        } else {
            CapRights::NONE
        };

        // If granted, record capability grant or policy allow in audit log
        let audit_seq = {
            let mut audit = AUDIT_LOG.lock();
            let event = if decision == ConsentDecision::Granted {
                AuditEventType::CapabilityGrant
            } else {
                AuditEventType::PolicyDeny
            };
            audit
                .record(
                    event,
                    self.records[idx].requesting_pid,
                    self.records[idx].resource_id,
                    self.records[idx].granted_rights,
                    timestamp,
                )
                .unwrap_or(0)
        };

        self.records[idx].audit_seq = audit_seq;

        // Deactivate isolated plane and release hardware input back to system
        self.surface.deactivate();

        Ok(self.records[idx])
    }

    /// Attempts to draw on the display plane from an untrusted process (INV-CON-001)
    pub fn attempt_untrusted_overlay(&self, caller_pid: ProcessId) -> Result<(), &'static str> {
        self.surface.verify_surface_access(caller_pid)
    }

    /// Attempts to inject synthetic input while a consent prompt is active (INV-CON-001)
    pub fn attempt_untrusted_input(&self, caller_pid: ProcessId) -> Result<(), &'static str> {
        self.surface.verify_input_injection(caller_pid)
    }

    /// Returns whether the isolated secure display surface is currently active
    pub fn is_prompt_active(&self) -> bool {
        self.surface.is_active()
    }

    /// Total number of consent records stored
    pub fn len(&self) -> usize {
        self.count
    }

    /// Retrieves a record by consent_id
    pub fn get_record(&self, consent_id: u64) -> Option<&ConsentRecord> {
        for i in 0..self.count {
            if self.records[i].consent_id == consent_id {
                return Some(&self.records[i]);
            }
        }
        None
    }
}
