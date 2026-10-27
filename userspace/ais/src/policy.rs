//! Ind OS Standing User Policy Client & Dry-Run Evaluation (SP-43)
//!
//! Standards:
//! - INV-POL-001: Standing policy immutability & revocability
//! - INV-POL-002: Deterministic conflict resolution
//! - INV-POL-003: Side-effect-free dry-run simulation
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::schema::ActionCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyLifecycleState {
    Active,
    Suspended,
    Revoked,
}

pub type PolicyStatus = PolicyLifecycleState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingPolicy {
    pub policy_id: u32,
    pub name: &'static str,
    pub category: ActionCategory,
    pub action_code: u32,
    pub state: PolicyLifecycleState,
    pub is_deny: bool,
    pub priority: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DryRunReport {
    pub would_allow: bool,
    pub firing_rule_id: Option<u32>,
    pub needs_consent: bool,
    pub conflicts_detected: u32,
}

pub const MAX_USER_POLICIES: usize = 16;

pub struct PolicyRegistry {
    policies: [Option<StandingPolicy>; MAX_USER_POLICIES],
    count: usize,
    next_id: u32,
}

impl PolicyRegistry {
    pub const fn new() -> Self {
        Self {
            policies: [None; MAX_USER_POLICIES],
            count: 0,
            next_id: 1001,
        }
    }

    pub fn register(&mut self, policy: StandingPolicy) -> Result<(), ()> {
        if self.count >= MAX_USER_POLICIES {
            return Err(());
        }
        self.policies[self.count] = Some(policy);
        self.count += 1;
        Ok(())
    }

    pub fn simulate_evaluation(&self, category: ActionCategory, action_code: u32) -> DryRunReport {
        let mut would_allow = false;
        let mut firing_id = None;
        let mut conflicts = 0;
        let mut saw_deny = false;

        for i in 0..self.count {
            if let Some(ref pol) = self.policies[i] {
                if pol.state == PolicyLifecycleState::Active
                    && pol.category == category
                    && (pol.action_code == 0xFFFF || pol.action_code == action_code)
                {
                    if pol.is_deny {
                        saw_deny = true;
                        firing_id = Some(pol.policy_id);
                        if would_allow {
                            conflicts += 1;
                        }
                    } else if !saw_deny {
                        would_allow = true;
                        firing_id = Some(pol.policy_id);
                    } else {
                        conflicts += 1;
                    }
                }
            }
        }

        DryRunReport {
            would_allow: would_allow && !saw_deny,
            firing_rule_id: firing_id,
            needs_consent: !would_allow && !saw_deny,
            conflicts_detected: conflicts,
        }
    }

    pub fn evaluate(&self, category: ActionCategory, action_code: u32) -> bool {
        self.simulate_evaluation(category, action_code).would_allow
    }
}
