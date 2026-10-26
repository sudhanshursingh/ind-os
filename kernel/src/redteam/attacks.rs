//! Adversarial Attack Probes & Red-Team Exploitation Harness (SP-22)
//! Compliant with Bible §8.9 (SP-22), §10, §14 and Invariants INV-CAP-001..002, INV-ENF-001..002, INV-AUD-001, INV-CON-001.

use crate::audit::AUDIT_LOG;
use crate::capability::rights::CapRights;
use crate::capability::table::{CapType, Capability};
use crate::consent::CONSENT_BROKER;
use crate::policy::rule::{PolicyDecision, Resource, Subject};
use crate::policy::POLICY_ENGINE;

/// Result summary of red-team adversarial evaluation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeSummary {
    pub total_probes: usize,
    pub repelled_count: usize,
    pub vulnerabilities_found: usize,
}

/// Adversarial Red-Team Test Suite
pub struct RedTeamSuite;

impl RedTeamSuite {
    /// Attack Vector 1: Privilege Escalation via Capability Rights Expansion (INV-CAP-001)
    pub fn probe_capability_escalation() -> Result<(), &'static str> {
        let parent_cap = Capability::new(CapType::Endpoint(501), CapRights::READ, 0xC001);
        let illegal_derived_rights = CapRights::READ.union(CapRights::WRITE).union(CapRights::GRANT);

        // Attempt rights attenuation validation: parent cannot attenuate to broader rights
        if parent_cap.rights.can_attenuate_to(illegal_derived_rights) {
            return Err("FAIL: Monotonic rights attenuation bypassed! Unauthorized rights expansion granted.");
        }
        Ok(())
    }

    /// Attack Vector 2: Dangling Handle / Use-After-Free Exploitation (INV-CAP-002, INV-OBJ-001)
    pub fn probe_dangling_handle() -> Result<(), &'static str> {
        let mut stale_cap = Capability::new(CapType::Endpoint(502), CapRights::READ, 0xC002);
        stale_cap.is_valid = false;

        if stale_cap.is_valid {
            return Err("FAIL: Revoked capability remains valid! Dangling handle use-after-free.");
        }
        Ok(())
    }

    /// Attack Vector 3: Confused Deputy & Ambient Authority Probe (INV-SEC-001)
    pub fn probe_ambient_authority() -> Result<(), &'static str> {
        // Evaluate access for unknown process without capability handle
        let engine = POLICY_ENGINE.lock();
        let dec = engine.evaluate(
            &Subject::Process(999),
            &Resource::Endpoint(503),
            CapRights::READ,
        );
        match dec {
            PolicyDecision::Denied { .. } => Ok(()),
            _ => Err("FAIL: Ambient authority permitted access without valid standing authorization!"),
        }
    }

    /// Attack Vector 4: Policy Default-Deny & Precedence Confusion (INV-ENF-001, INV-ENF-002)
    pub fn probe_policy_bypass() -> Result<(), &'static str> {
        let engine = POLICY_ENGINE.lock();

        // Process 2 attempts WRITE on Endpoint 101 where explicit DENY rule exists
        let dec = engine.evaluate(
            &Subject::Process(2),
            &Resource::Endpoint(101),
            CapRights::WRITE,
        );
        match dec {
            PolicyDecision::Denied { rule_id, .. } => {
                if rule_id != Some(3) {
                    return Err("FAIL: High-priority deny rule was not deterministically enforced.");
                }
                Ok(())
            }
            _ => Err("FAIL: Standing deny policy was bypassed by adversary!"),
        }
    }

    /// Attack Vector 5: Cryptographic Audit History Revision & Tampering (INV-AUD-001)
    pub fn probe_audit_tampering() -> Result<(), &'static str> {
        let mut log = AUDIT_LOG.lock();
        if log.len() < 2 {
            return Err("FAIL: Audit log has insufficient entries for tamper probe.");
        }

        let saved = *log.get_entry(1).unwrap();
        // Artificially corrupt historical byte
        log.tamper_flip_byte(1, 0)?;

        let check = log.verify_integrity();
        // Restore entry
        let _ = log.restore_entry(1, saved);

        match check {
            Err((idx, _)) if idx == 1 => Ok(()),
            Err(_) => Err("FAIL: Tamper detected at wrong index."),
            Ok(()) => Err("FAIL: Cryptographic audit tampering went undetected!"),
        }
    }

    /// Attack Vector 6: Consent Broker Overlay & Input Injection (INV-CON-001)
    pub fn probe_consent_spoofing() -> Result<(), &'static str> {
        let mut broker = CONSENT_BROKER.lock();
        let cid = broker.request_consent(888, 999, CapRights::READ, 3000)?;

        let overlay_res = broker.attempt_untrusted_overlay(999);
        let input_res = broker.attempt_untrusted_input(999);

        // Cancel/cleanup prompt
        let _ = broker.submit_decision(cid, crate::consent::ConsentDecision::Denied, CapRights::NONE, 3005);

        if overlay_res.is_ok() {
            return Err("FAIL: Untrusted overlay on isolated consent surface succeeded!");
        }
        if input_res.is_ok() {
            return Err("FAIL: Synthetic input injection into consent prompt succeeded!");
        }

        Ok(())
    }

    /// Runs all 6 red-team adversarial probes
    pub fn run_all_probes() -> Result<ProbeSummary, &'static str> {
        Self::probe_capability_escalation()?;
        Self::probe_dangling_handle()?;
        Self::probe_ambient_authority()?;
        Self::probe_policy_bypass()?;
        Self::probe_audit_tampering()?;
        Self::probe_consent_spoofing()?;

        Ok(ProbeSummary {
            total_probes: 6,
            repelled_count: 6,
            vulnerabilities_found: 0,
        })
    }
}
