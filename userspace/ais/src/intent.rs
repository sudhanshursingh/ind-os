//! Ind OS AIS Intent Broker (REQ-AI-102, REQ-AI-003)
//!
//! Standards:
//! - REQ-AI-102: Intent broker (NL/system-event -> typed proposal)
//! - REQ-AI-003: Pre-routing proposal audit logging
//! - INV-INTENT-001: Typed proposal schema enforcement
//! - INV-INTENT-002: Pre-routing proposal audit logging
//! - INV-INTENT-003: Advisory-only intent posture
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::schema::{ActionCategory, SchemaRegistry};
use indus_logd::{LogBus, LogLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentSource {
    NaturalLanguage,
    SystemEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedProposal {
    pub proposal_id: u32,
    pub source: IntentSource,
    pub category: ActionCategory,
    pub action_code: u32,
    pub target_id: u32,
    pub confidence_score: u8, // 0..100
    pub required_rights: u32,
    pub pre_logged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentError {
    EmptyIntent,
    UnrecognizedIntent,
    UnregisteredSchema,
    PreRoutingLogFailed,
}

pub struct IntentBroker {
    registry: SchemaRegistry,
    next_proposal_id: u32,
}

impl IntentBroker {
    pub fn new(registry: SchemaRegistry) -> Self {
        Self {
            registry,
            next_proposal_id: 1000,
        }
    }

    pub fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut SchemaRegistry {
        &mut self.registry
    }

    /// Translates an incoming intent and immutably logs it pre-routing (REQ-AI-003).
    pub fn process_intent(
        &mut self,
        source: IntentSource,
        intent_text: &str,
        log_bus: &mut LogBus,
    ) -> Result<TypedProposal, IntentError> {
        let trimmed = intent_text.trim();
        if trimmed.is_empty() {
            return Err(IntentError::EmptyIntent);
        }

        // Pattern mapping for baseline intents
        let (category, action_code, target_id, confidence) = if trimmed.contains("bluetooth") {
            (ActionCategory::DeviceControl, 101, 1, 95)
        } else if trimmed.contains("brightness") {
            (ActionCategory::SystemSetting, 201, 50, 90)
        } else if trimmed.contains("notify") || trimmed.contains("alert") {
            (ActionCategory::Notification, 301, 10, 85)
        } else if trimmed.contains("document") || trimmed.contains("read") {
            (ActionCategory::FileOperation, 401, 2, 88)
        } else {
            return Err(IntentError::UnrecognizedIntent);
        };

        // 1. Schema lookup & validation (INV-INTENT-001)
        let schema = self.registry.lookup(category, action_code)
            .ok_or(IntentError::UnregisteredSchema)?;

        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        // 2. Pre-routing audit log (REQ-AI-003, INV-INTENT-002)
        // Log to unified log bus BEFORE returning/routing to ENF
        log_bus.ingest(
            100, // mock timestamp
            LogLevel::Info,
            1, // subsystem: AIS
            proposal_id,
            schema.name,
        );

        let proposal = TypedProposal {
            proposal_id,
            source,
            category,
            action_code,
            target_id,
            confidence_score: confidence,
            required_rights: schema.required_rights,
            pre_logged: true,
        };

        Ok(proposal)
    }
}
