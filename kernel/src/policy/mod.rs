//! Enforcement Layer (ENF) Security Policy Engine
//! Implements declarative policy rule language, deterministic precedence resolution (INV-ENF-002),
//! default-deny evaluation (INV-ENF-001), and dual-check capability access control per Bible §6.3 / §8.9.

pub mod engine;
pub mod rule;

pub use engine::{PolicyEngine, MAX_POLICY_RULES};
pub use rule::{PolicyAction, PolicyDecision, PolicyRule, Resource, Subject};

use crate::sync::Spinlock;

/// Global singleton instance of the ENF Policy Engine
pub static POLICY_ENGINE: Spinlock<PolicyEngine> = Spinlock::new(PolicyEngine::new());
