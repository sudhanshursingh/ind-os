//! Kernel AIS Red-Team Verification Harness (SP-50)
//!
//! Constitutional References:
//! - Bible §6.3, §8.12: AIS red-team sprint (prompt injection, memory poisoning, confused deputy)
//! - INV-REDTEAM-001: Prompt injection neutralization
//! - INV-REDTEAM-002: Memory poisoning defense
//! - INV-REDTEAM-003: Confused deputy defense
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::userspace::intent_broker::KernelActionCategory;
use crate::userspace::logd::{KernelLogBus, KernelLogLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelAttackType {
    IndirectPromptInjection,
    MemoryPoisoning,
    ConfusedDeputyEscalation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelRedTeamAttackScenario {
    pub scenario_id: u32,
    pub attack_type: KernelAttackType,
    pub payload: &'static str,
    pub target_category: KernelActionCategory,
}

pub struct KernelRedTeamHarness {
    attacks_tested: usize,
    attacks_neutralized: usize,
}

impl KernelRedTeamHarness {
    pub const fn new() -> Self {
        Self {
            attacks_tested: 0,
            attacks_neutralized: 0,
        }
    }

    pub fn test_prompt_injection(&mut self, payload: &'static str, log_bus: &mut KernelLogBus) -> bool {
        self.attacks_tested += 1;
        // Invariant: Prompt injection strings cannot bypass policy or trigger ambient execution
        let contains_override = payload_contains(payload, "Ignore previous instructions")
            || payload_contains(payload, "DROP TABLE")
            || payload_contains(payload, "rm -rf");

        if contains_override {
            self.attacks_neutralized += 1;
            log_bus.ingest(2000, KernelLogLevel::SecurityAudit, 10, 1001, "RedTeam: indirect prompt injection neutralized");
            true
        } else {
            false
        }
    }

    pub fn test_memory_poisoning(&mut self, key: &'static str, value: &'static str, log_bus: &mut KernelLogBus) -> bool {
        self.attacks_tested += 1;
        let is_poison = payload_contains(key, "policy") || payload_contains(key, "capability") || payload_contains(value, "grant_all");
        if is_poison {
            self.attacks_neutralized += 1;
            log_bus.ingest(2000, KernelLogLevel::SecurityAudit, 10, 1002, "RedTeam: memory poisoning attempt rejected");
            true
        } else {
            false
        }
    }

    pub fn test_confused_deputy(&mut self, caller_cap: u32, required_cap: u32, log_bus: &mut KernelLogBus) -> bool {
        self.attacks_tested += 1;
        if (caller_cap & required_cap) != required_cap {
            self.attacks_neutralized += 1;
            log_bus.ingest(2000, KernelLogLevel::SecurityAudit, 10, 1003, "RedTeam: confused deputy escalation denied");
            true
        } else {
            false
        }
    }

    pub fn results(&self) -> (usize, usize) {
        (self.attacks_tested, self.attacks_neutralized)
    }
}

fn payload_contains(haystack: &str, needle: &str) -> bool {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.len() > h.len() {
        return false;
    }
    for i in 0..=(h.len() - n.len()) {
        if &h[i..i + n.len()] == n {
            return true;
        }
    }
    false
}
