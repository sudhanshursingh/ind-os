//! Enforcement Layer (ENF) Policy Engine v1
//! Compliant with Bible §6.3, §8.9 (SP-19) and Machine Invariants:
//! - INV-ENF-001: Default-Deny Security Model
//! - INV-ENF-002: Deterministic Precedence & Versioned Rule Resolution

use crate::capability::rights::CapRights;
use crate::capability::table::{CapType, Capability};
use crate::policy::rule::{PolicyAction, PolicyDecision, PolicyRule, Resource, Subject};
use crate::syscall::{EBADF, ENOMEM, EPERM};
use crate::task::ProcessId;

pub const MAX_POLICY_RULES: usize = 64;

/// ENF Policy Engine evaluating declarative rules on capability accesses
pub struct PolicyEngine {
    pub rules: [PolicyRule; MAX_POLICY_RULES],
    pub active_version: u32,
    pub rule_count: usize,
    pub default_deny: bool,
}

impl PolicyEngine {
    pub const fn new() -> Self {
        Self {
            rules: [PolicyRule::empty(); MAX_POLICY_RULES],
            active_version: 1,
            rule_count: 0,
            default_deny: true, // INV-ENF-001: Always default-deny
        }
    }

    /// Adds a policy rule to the engine
    pub fn add_rule(&mut self, rule: PolicyRule) -> Result<u32, i64> {
        for slot in self.rules.iter_mut() {
            if !slot.is_active {
                *slot = rule;
                self.rule_count += 1;
                return Ok(rule.id);
            }
        }
        Err(ENOMEM)
    }

    /// Removes a policy rule by id
    pub fn remove_rule(&mut self, id: u32) -> Result<(), i64> {
        for slot in self.rules.iter_mut() {
            if slot.is_active && slot.id == id {
                *slot = PolicyRule::empty();
                self.rule_count = self.rule_count.saturating_sub(1);
                return Ok(());
            }
        }
        Err(EBADF)
    }

    /// Loads a declarative ruleset string, wiping existing active rules and setting version
    pub fn load_declarations(&mut self, version: u32, declarations: &[&str]) -> Result<usize, i64> {
        self.rules = [PolicyRule::empty(); MAX_POLICY_RULES];
        self.rule_count = 0;
        self.active_version = version;

        let mut loaded = 0;
        for decl in declarations {
            let rule = PolicyRule::parse_declaration(decl)?;
            self.add_rule(rule)?;
            loaded += 1;
        }
        Ok(loaded)
    }

    /// Evaluates policy for a subject, resource, and requested rights (INV-ENF-001, INV-ENF-002)
    pub fn evaluate(
        &self,
        subject: &Subject,
        resource: &Resource,
        requested_rights: CapRights,
    ) -> PolicyDecision {
        let mut highest_deny: Option<(u16, u32)> = None;
        let mut highest_allow: Option<(u16, u32)> = None;
        let mut highest_consent: Option<(u16, u32)> = None;

        for rule in self.rules.iter() {
            if rule.is_active && rule.matches(subject, resource, requested_rights) {
                match rule.action {
                    PolicyAction::Deny => {
                        if highest_deny.is_none() || rule.priority > highest_deny.unwrap().0 {
                            highest_deny = Some((rule.priority, rule.id));
                        }
                    }
                    PolicyAction::Allow => {
                        if highest_allow.is_none() || rule.priority > highest_allow.unwrap().0 {
                            highest_allow = Some((rule.priority, rule.id));
                        }
                    }
                    PolicyAction::RequireConsent => {
                        if highest_consent.is_none() || rule.priority > highest_consent.unwrap().0 {
                            highest_consent = Some((rule.priority, rule.id));
                        }
                    }
                }
            }
        }

        // Deterministic Precedence Resolution (INV-ENF-002):
        // 1. Explicit Deny overrides Allow and Consent if Deny priority >= Allow/Consent priority
        if let Some((deny_prio, deny_id)) = highest_deny {
            let allow_prio = highest_allow.map(|(p, _)| p).unwrap_or(0);
            let consent_prio = highest_consent.map(|(p, _)| p).unwrap_or(0);
            if deny_prio >= allow_prio && deny_prio >= consent_prio {
                return PolicyDecision::Denied {
                    rule_id: Some(deny_id),
                    reason: "Explicit Deny Rule Precedence (INV-ENF-002)",
                };
            }
        }

        // 2. Consent takes precedence if higher than Allow
        if let Some((consent_prio, consent_id)) = highest_consent {
            let allow_prio = highest_allow.map(|(p, _)| p).unwrap_or(0);
            if consent_prio > allow_prio {
                return PolicyDecision::NeedsConsent { rule_id: consent_id };
            }
        }

        // 3. Allow rule
        if let Some((_, allow_id)) = highest_allow {
            return PolicyDecision::Allowed { rule_id: allow_id };
        }

        // 4. Default-Deny Fallback (INV-ENF-001)
        PolicyDecision::Denied {
            rule_id: None,
            reason: "Default-Deny: No matching allow rule (INV-ENF-001)",
        }
    }

    /// Dual-check verification: validates both capability token possession AND standing ENF policy authorization
    pub fn check_capability_access(
        &self,
        pid: ProcessId,
        cap: &Capability,
        requested_rights: CapRights,
    ) -> Result<(), i64> {
        // Step 1: Token Possession Check
        if !cap.is_valid {
            return Err(EBADF);
        }
        if !cap.rights.contains(requested_rights) {
            return Err(EPERM);
        }

        // Step 2: Policy Engine Enforcement Check (Standing Policy)
        let resource = match cap.cap_type {
            CapType::Object(kobj_id) => Resource::Object(kobj_id),
            CapType::Endpoint(ep) => Resource::Endpoint(ep),
            CapType::MemoryRegion { base, size } => Resource::MemoryRegion { base, size },
            _ => Resource::Any,
        };

        match self.evaluate(&Subject::Process(pid), &resource, requested_rights) {
            PolicyDecision::Allowed { .. } => Ok(()),
            PolicyDecision::Denied { .. } => Err(EPERM),
            PolicyDecision::NeedsConsent { .. } => Err(EPERM),
        }
    }

    /// Returns the number of currently active rules
    pub fn active_count(&self) -> usize {
        self.rule_count
    }
}
