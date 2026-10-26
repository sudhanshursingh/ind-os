//! # Ind OS Intent ABI v1.0 Skeleton
//!
//! Enforces strongly-typed intent declaration and validation across the Semantic Gateway.

use ind_core_types::{ActionTier, DenialCode, SessionId};
use serde::{Deserialize, Serialize};

pub const ABI_VERSION_CURRENT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEnvelope {
    pub abi_version: u32,
    pub session_id: [u8; 32],
    pub timestamp_ns: u64,
    pub target_action: String,
    pub parameters_json: String,
    pub declared_tier: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentValidationResult {
    Authorized {
        computed_tier: ActionTier,
        target_action: String,
    },
    Denied {
        reason: DenialCode,
        detail: &'static str,
    },
}

pub fn validate_intent_envelope(envelope: &IntentEnvelope) -> IntentValidationResult {
    if envelope.abi_version != ABI_VERSION_CURRENT {
        return IntentValidationResult::Denied {
            reason: DenialCode::InvalidSchema,
            detail: "Unsupported Intent ABI version",
        };
    }

    if envelope.target_action.is_empty() {
        return IntentValidationResult::Denied {
            reason: DenialCode::InvalidSchema,
            detail: "target_action cannot be empty",
        };
    }

    // Default classification rule based on PRD §5.1 / §6.1
    let tier = match envelope.target_action.as_str() {
        "device.torch.toggle" | "device.screen.brightness" | "system.time.query" => ActionTier::Green,
        "telephony.sms.send" | "app.launch" | "contacts.query" => ActionTier::Amber,
        "financial.upi.pay" | "device.factory_reset" | "security.lock.bypass" => ActionTier::Red,
        _ => ActionTier::Amber, // Default fail-safe tier
    };

    IntentValidationResult::Authorized {
        computed_tier: tier,
        target_action: envelope.target_action.clone(),
    }
}
