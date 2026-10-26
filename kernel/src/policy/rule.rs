//! Declarative Policy Rule Language and Structures (ENF Policy Engine v1)
//! Compliant with Bible §6.3, §8.9 (SP-19) and Machine Invariants:
//! - INV-ENF-001: Default Deny
//! - INV-ENF-002: Deterministic Precedence Resolution

use crate::capability::rights::CapRights;
use crate::object::KObjectId;
use crate::syscall::EINVAL;
use crate::task::ProcessId;

/// Subject entity requesting access
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    Any,
    Process(ProcessId),
    Role(u32), // 1 = Core, 2 = Driver, 3 = AIS, 4 = App
}

/// Target kernel resource
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Any,
    Endpoint(u32),
    MemoryRegion { base: usize, size: usize },
    Object(KObjectId),
    Syscall(u32),
}

/// Enforcement action determined by rule
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyAction {
    Deny,
    Allow,
    RequireConsent,
}

/// Decision returned by policy evaluation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyDecision {
    Allowed { rule_id: u32 },
    Denied { rule_id: Option<u32>, reason: &'static str },
    NeedsConsent { rule_id: u32 },
}

/// Declarative, versioned policy rule
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyRule {
    pub id: u32,
    pub version: u32,
    pub priority: u16, // Higher number = higher precedence
    pub subject: Subject,
    pub resource: Resource,
    pub rights: CapRights,
    pub action: PolicyAction,
    pub is_active: bool,
}

impl PolicyRule {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            version: 0,
            priority: 0,
            subject: Subject::Any,
            resource: Resource::Any,
            rights: CapRights::NONE,
            action: PolicyAction::Deny,
            is_active: false,
        }
    }

    pub fn allow(
        id: u32,
        version: u32,
        priority: u16,
        subject: Subject,
        resource: Resource,
        rights: CapRights,
    ) -> Self {
        Self {
            id,
            version,
            priority,
            subject,
            resource,
            rights,
            action: PolicyAction::Allow,
            is_active: true,
        }
    }

    pub fn deny(
        id: u32,
        version: u32,
        priority: u16,
        subject: Subject,
        resource: Resource,
        rights: CapRights,
    ) -> Self {
        Self {
            id,
            version,
            priority,
            subject,
            resource,
            rights,
            action: PolicyAction::Deny,
            is_active: true,
        }
    }

    pub fn require_consent(
        id: u32,
        version: u32,
        priority: u16,
        subject: Subject,
        resource: Resource,
        rights: CapRights,
    ) -> Self {
        Self {
            id,
            version,
            priority,
            subject,
            resource,
            rights,
            action: PolicyAction::RequireConsent,
            is_active: true,
        }
    }

    /// Matches this rule against a subject, resource, and requested rights
    pub fn matches(&self, subject: &Subject, resource: &Resource, requested_rights: CapRights) -> bool {
        if !self.is_active {
            return false;
        }

        // 1. Subject match
        let subject_matches = match (self.subject, *subject) {
            (Subject::Any, _) => true,
            (Subject::Process(p1), Subject::Process(p2)) => p1 == p2,
            (Subject::Role(r1), Subject::Role(r2)) => r1 == r2,
            _ => false,
        };
        if !subject_matches {
            return false;
        }

        // 2. Resource match
        let resource_matches = match (self.resource, *resource) {
            (Resource::Any, _) => true,
            (Resource::Endpoint(e1), Resource::Endpoint(e2)) => e1 == e2,
            (Resource::Object(o1), Resource::Object(o2)) => o1 == o2,
            (Resource::Syscall(s1), Resource::Syscall(s2)) => s1 == s2,
            (
                Resource::MemoryRegion { base: b1, size: s1 },
                Resource::MemoryRegion { base: b2, size: s2 },
            ) => b1 == b2 && s1 == s2,
            _ => false,
        };
        if !resource_matches {
            return false;
        }

        // 3. Rights match (rule rights must cover requested rights for Allow; for Deny, any intersection triggers)
        match self.action {
            PolicyAction::Allow | PolicyAction::RequireConsent => {
                self.rights.contains(requested_rights)
            }
            PolicyAction::Deny => {
                // If rule denies any requested right
                self.rights.contains(requested_rights) || (self.rights.bits() & requested_rights.bits() != 0)
            }
        }
    }

    /// Parses a declarative, human-readable rule declaration
    /// Format: "ALLOW|DENY|CONSENT sub:pid|role|any res:ep|obj|any rights:READ|WRITE|GRANT|CALL prio:<num> id:<num> v:<num>"
    pub fn parse_declaration(decl: &str) -> Result<Self, i64> {
        let mut tokens = decl.split_whitespace();
        let action_str = tokens.next().ok_or(EINVAL)?;
        let action = match action_str {
            "ALLOW" => PolicyAction::Allow,
            "DENY" => PolicyAction::Deny,
            "CONSENT" => PolicyAction::RequireConsent,
            _ => return Err(EINVAL),
        };

        let mut subject = Subject::Any;
        let mut resource = Resource::Any;
        let mut rights = CapRights::NONE;
        let mut priority = 10u16;
        let mut id = 1u32;
        let mut version = 1u32;

        for token in tokens {
            if let Some(s) = token.strip_prefix("sub:") {
                if s == "any" {
                    subject = Subject::Any;
                } else if let Some(pid_str) = s.strip_prefix("pid:") {
                    let pid: u64 = parse_u64(pid_str)?;
                    subject = Subject::Process(pid as ProcessId);
                } else if let Some(role_str) = s.strip_prefix("role:") {
                    let role: u64 = parse_u64(role_str)?;
                    subject = Subject::Role(role as u32);
                }
            } else if let Some(r) = token.strip_prefix("res:") {
                if r == "any" {
                    resource = Resource::Any;
                } else if let Some(ep_str) = r.strip_prefix("ep:") {
                    let ep: u64 = parse_u64(ep_str)?;
                    resource = Resource::Endpoint(ep as u32);
                } else if let Some(sys_str) = r.strip_prefix("sys:") {
                    let sys: u64 = parse_u64(sys_str)?;
                    resource = Resource::Syscall(sys as u32);
                }
            } else if let Some(rt) = token.strip_prefix("rights:") {
                rights = parse_rights(rt)?;
            } else if let Some(prio_str) = token.strip_prefix("prio:") {
                priority = parse_u64(prio_str)? as u16;
            } else if let Some(id_str) = token.strip_prefix("id:") {
                id = parse_u64(id_str)? as u32;
            } else if let Some(v_str) = token.strip_prefix("v:") {
                version = parse_u64(v_str)? as u32;
            }
        }

        Ok(Self {
            id,
            version,
            priority,
            subject,
            resource,
            rights,
            action,
            is_active: true,
        })
    }
}

fn parse_u64(s: &str) -> Result<u64, i64> {
    let mut val: u64 = 0;
    if s.is_empty() {
        return Err(EINVAL);
    }
    for b in s.bytes() {
        if b >= b'0' && b <= b'9' {
            val = val.checked_mul(10).ok_or(EINVAL)?;
            val = val.checked_add((b - b'0') as u64).ok_or(EINVAL)?;
        } else {
            return Err(EINVAL);
        }
    }
    Ok(val)
}

fn parse_rights(s: &str) -> Result<CapRights, i64> {
    let mut r = CapRights::NONE;
    for part in s.split('|') {
        match part {
            "READ" => r = r.union(CapRights::READ),
            "WRITE" => r = r.union(CapRights::WRITE),
            "GRANT" => r = r.union(CapRights::GRANT),
            "CALL" => r = r.union(CapRights::CALL),
            "ALL" => r = CapRights::ALL,
            "NONE" => {},
            _ => return Err(EINVAL),
        }
    }
    Ok(r)
}
