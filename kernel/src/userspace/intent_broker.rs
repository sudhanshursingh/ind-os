//! Kernel Intent Broker & Schema Registry Verification Harness (SP-42)
//!
//! Constitutional References:
//! - REQ-AI-102: Intent broker (NL/system-event -> typed proposal)
//! - REQ-AI-003: Pre-routing proposal audit logging
//! - INV-INTENT-001: Typed proposal schema enforcement
//! - INV-INTENT-002: Pre-routing proposal audit logging
//! - INV-INTENT-003: Advisory-only intent posture
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const KERNEL_MAX_SCHEMAS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelActionCategory {
    DeviceControl = 1,
    FileOperation = 2,
    SystemSetting = 3,
    Notification = 4,
    NetworkTransfer = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSchemaDescriptor {
    pub category: KernelActionCategory,
    pub action_code: u32,
    pub name: &'static str,
    pub required_rights: u32,
}

pub struct KernelSchemaRegistry {
    schemas: [Option<KernelSchemaDescriptor>; KERNEL_MAX_SCHEMAS],
    count: usize,
}

impl KernelSchemaRegistry {
    pub const fn new() -> Self {
        Self {
            schemas: [None; KERNEL_MAX_SCHEMAS],
            count: 0,
        }
    }

    pub fn register(&mut self, descriptor: KernelSchemaDescriptor) -> Result<(), ()> {
        if self.count >= KERNEL_MAX_SCHEMAS {
            return Err(());
        }
        self.schemas[self.count] = Some(descriptor);
        self.count += 1;
        Ok(())
    }

    pub fn lookup(&self, category: KernelActionCategory, action_code: u32) -> Option<KernelSchemaDescriptor> {
        for i in 0..self.count {
            if let Some(s) = self.schemas[i] {
                if s.category == category && s.action_code == action_code {
                    return Some(s);
                }
            }
        }
        None
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelTypedProposal {
    pub proposal_id: u32,
    pub category: KernelActionCategory,
    pub action_code: u32,
    pub target_id: u32,
    pub confidence_score: u8,
    pub required_rights: u32,
    pub pre_logged: bool,
}

pub struct KernelIntentBroker {
    registry: KernelSchemaRegistry,
    next_proposal_id: u32,
}

impl KernelIntentBroker {
    pub fn new(registry: KernelSchemaRegistry) -> Self {
        Self {
            registry,
            next_proposal_id: 2000,
        }
    }

    pub fn parse_and_propose(
        &mut self,
        intent_text: &str,
        log_bus: &mut crate::userspace::KernelLogBus,
    ) -> Result<KernelTypedProposal, ()> {
        let (category, action_code, target_id, confidence) = if intent_text.contains("bluetooth") {
            (KernelActionCategory::DeviceControl, 101, 1, 95)
        } else if intent_text.contains("brightness") {
            (KernelActionCategory::SystemSetting, 201, 50, 90)
        } else {
            return Err(());
        };

        let schema = self.registry.lookup(category, action_code).ok_or(())?;

        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        // Pre-routing audit log (REQ-AI-003, INV-INTENT-002)
        log_bus.ingest(
            150,
            crate::userspace::KernelLogLevel::Info,
            1, // Subsystem AIS
            proposal_id,
            schema.name,
        );

        Ok(KernelTypedProposal {
            proposal_id,
            category,
            action_code,
            target_id,
            confidence_score: confidence,
            required_rights: schema.required_rights,
            pre_logged: true,
        })
    }
}
