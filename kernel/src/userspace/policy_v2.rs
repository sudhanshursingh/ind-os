//! Kernel Policy Engine v2 Verification Harness (SP-43)
//!
//! Constitutional References:
//! - Bible §8.12: Policy engine v2 (standing user policies, conflict resolution, dry-run simulation)
//! - INV-POL-001: Standing policy immutability & revocability
//! - INV-POL-002: Deterministic conflict resolution
//! - INV-POL-003: Side-effect-free dry-run simulation
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::userspace::intent_broker::KernelActionCategory;

pub const MAX_STANDING_POLICIES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPolicyLifecycleState {
    Active,
    Suspended,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelStandingPolicy {
    pub policy_id: u32,
    pub name: &'static str,
    pub category: KernelActionCategory,
    pub action_code: u32,
    pub state: KernelPolicyLifecycleState,
    pub is_deny: bool,
    pub priority: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelDryRunReport {
    pub would_allow: bool,
    pub firing_rule_id: Option<u32>,
    pub needs_consent: bool,
    pub conflicts_detected: u32,
}

pub struct KernelPolicyEngineV2 {
    policies: [Option<KernelStandingPolicy>; MAX_STANDING_POLICIES],
    count: usize,
    next_id: u32,
}

impl KernelPolicyEngineV2 {
    pub const fn new() -> Self {
        Self {
            policies: [None; MAX_STANDING_POLICIES],
            count: 0,
            next_id: 3001,
        }
    }

    pub fn register_policy(
        &mut self,
        name: &'static str,
        category: KernelActionCategory,
        action_code: u32,
        is_deny: bool,
        priority: u16,
    ) -> Result<u32, ()> {
        if self.count >= MAX_STANDING_POLICIES {
            return Err(());
        }
        let id = self.next_id;
        self.next_id += 1;

        self.policies[self.count] = Some(KernelStandingPolicy {
            policy_id: id,
            name,
            category,
            action_code,
            state: KernelPolicyLifecycleState::Active,
            is_deny,
            priority,
        });
        self.count += 1;
        Ok(id)
    }

    pub fn revoke_policy(&mut self, policy_id: u32) -> Result<(), ()> {
        for i in 0..self.count {
            if let Some(ref mut p) = self.policies[i] {
                if p.policy_id == policy_id {
                    p.state = KernelPolicyLifecycleState::Revoked;
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn suspend_policy(&mut self, policy_id: u32) -> Result<(), ()> {
        for i in 0..self.count {
            if let Some(ref mut p) = self.policies[i] {
                if p.policy_id == policy_id {
                    p.state = KernelPolicyLifecycleState::Suspended;
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn inspect_policy(&self, policy_id: u32) -> Option<&KernelStandingPolicy> {
        for i in 0..self.count {
            if let Some(ref p) = self.policies[i] {
                if p.policy_id == policy_id {
                    return Some(p);
                }
            }
        }
        None
    }

    /// Evaluates action under conflict resolution rules (INV-POL-002).
    pub fn evaluate(&self, category: KernelActionCategory, action_code: u32) -> bool {
        let report = self.simulate_dry_run(category, action_code);
        report.would_allow
    }

    /// Side-effect-free dry-run simulation of a prospective action proposal (INV-POL-003).
    pub fn simulate_dry_run(&self, category: KernelActionCategory, action_code: u32) -> KernelDryRunReport {
        let mut highest_deny: Option<(u16, u32)> = None;
        let mut highest_allow: Option<(u16, u32)> = None;
        let mut matching_rules = 0u32;

        for i in 0..self.count {
            if let Some(ref p) = self.policies[i] {
                // Only consider Active policies (Suspended/Revoked are ignored per INV-POL-001)
                if p.state == KernelPolicyLifecycleState::Active
                    && p.category == category
                    && p.action_code == action_code
                {
                    matching_rules += 1;
                    if p.is_deny {
                        if highest_deny.is_none() || p.priority > highest_deny.unwrap().0 {
                            highest_deny = Some((p.priority, p.policy_id));
                        }
                    } else {
                        if highest_allow.is_none() || p.priority > highest_allow.unwrap().0 {
                            highest_allow = Some((p.priority, p.policy_id));
                        }
                    }
                }
            }
        }

        let conflicts = if highest_deny.is_some() && highest_allow.is_some() { 1 } else { 0 };

        // Conflict resolution: explicit deny overrides allow if deny_prio >= allow_prio (INV-POL-002)
        if let Some((deny_prio, deny_id)) = highest_deny {
            let allow_prio = highest_allow.map(|(p, _)| p).unwrap_or(0);
            if deny_prio >= allow_prio {
                return KernelDryRunReport {
                    would_allow: false,
                    firing_rule_id: Some(deny_id),
                    needs_consent: false,
                    conflicts_detected: conflicts,
                };
            }
        }

        if let Some((_, allow_id)) = highest_allow {
            return KernelDryRunReport {
                would_allow: true,
                firing_rule_id: Some(allow_id),
                needs_consent: false,
                conflicts_detected: conflicts,
            };
        }

        // Default-deny if no rule matches
        KernelDryRunReport {
            would_allow: false,
            firing_rule_id: None,
            needs_consent: true,
            conflicts_detected: 0,
        }
    }
}
