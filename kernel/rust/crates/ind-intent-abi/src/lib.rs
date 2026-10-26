//! # Ind OS Intent ABI v1.0 Reference Implementation & Validator
//!
//! Enforces:
//! 1. Formal JSON envelope parsing and structural validation.
//! 2. Autonomous ActionTier computation (ignoring AI's self-reported `declared_tier`).
//! 3. Taint tracking invariants (rejecting `UNTRUSTED_SCREEN_TEXT` on sensitive capabilities).
//! 4. 4 canonical decisions: `ALLOW`, `ALLOW_WITH_CONFIRMATION`, `DENY`, `DENY_AND_AUDIT`.

use ind_core_types::{ActionTier, DenialCode};
use serde::{Deserialize, Serialize};

pub const ABI_VERSION_CURRENT: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayDecision {
    Allow,
    AllowWithConfirmation,
    Deny,
    DenyAndAudit,
}

impl GatewayDecision {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::AllowWithConfirmation => "ALLOW_WITH_CONFIRMATION",
            Self::Deny => "DENY",
            Self::DenyAndAudit => "DENY_AND_AUDIT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionItem {
    pub action_type: String,
    pub target: String,
    pub params: serde_json::Value,
    pub data_taint_labels: Vec<String>,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserFeedback {
    #[serde(default)]
    pub tts_text: Option<String>,
    #[serde(default)]
    pub visual_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEnvelope {
    pub intent_id: String,
    pub timestamp_ms: i64,
    pub session_id: String,
    pub declared_tier: String,
    pub actions: Vec<ActionItem>,
    #[serde(default)]
    pub user_feedback: Option<UserFeedback>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationOutcome {
    pub decision: GatewayDecision,
    pub computed_tier: Option<ActionTier>,
    pub reason: Option<DenialCode>,
    pub detail: String,
}

fn is_valid_uuid(s: &str) -> bool {
    if s.len() != 36 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
        return false;
    }
    bytes.iter().enumerate().all(|(i, &b)| {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            true
        } else {
            b.is_ascii_hexdigit()
        }
    })
}

fn is_valid_capability_uri(s: &str) -> bool {
    if !s.starts_with("cap:") || s.len() < 5 || s.len() > 256 {
        return false;
    }
    s[4..].bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.' || b == b'-')
}

pub fn validate_intent_envelope_full(envelope: &IntentEnvelope) -> ValidationOutcome {
    // 1. Structural checks
    if !is_valid_uuid(&envelope.intent_id) {
        return ValidationOutcome {
            decision: GatewayDecision::Deny,
            computed_tier: None,
            reason: Some(DenialCode::InvalidSchema),
            detail: "Invalid intent_id UUID".into(),
        };
    }

    if !is_valid_uuid(&envelope.session_id) {
        return ValidationOutcome {
            decision: GatewayDecision::Deny,
            computed_tier: None,
            reason: Some(DenialCode::InvalidSchema),
            detail: "Invalid session_id UUID".into(),
        };
    }

    if envelope.timestamp_ms <= 0 {
        return ValidationOutcome {
            decision: GatewayDecision::Deny,
            computed_tier: None,
            reason: Some(DenialCode::InvalidSchema),
            detail: "timestamp_ms must be positive".into(),
        };
    }

    if !["GREEN", "AMBER", "RED"].contains(&envelope.declared_tier.as_str()) {
        return ValidationOutcome {
            decision: GatewayDecision::Deny,
            computed_tier: None,
            reason: Some(DenialCode::InvalidSchema),
            detail: "Unknown declared_tier".into(),
        };
    }

    if envelope.actions.is_empty() || envelope.actions.len() > 16 {
        return ValidationOutcome {
            decision: GatewayDecision::Deny,
            computed_tier: None,
            reason: Some(DenialCode::InvalidSchema),
            detail: "actions length must be between 1 and 16".into(),
        };
    }

    // 2. Validate action items, compute true tier, and inspect taint
    let mut max_tier = ActionTier::Green;

    for act in &envelope.actions {
        if act.target.is_empty() || act.target.len() > 256 {
            return ValidationOutcome {
                decision: GatewayDecision::Deny,
                computed_tier: None,
                reason: Some(DenialCode::InvalidSchema),
                detail: "Invalid target length".into(),
            };
        }

        for cap in &act.required_capabilities {
            if !is_valid_capability_uri(cap) {
                return ValidationOutcome {
                    decision: GatewayDecision::Deny,
                    computed_tier: None,
                    reason: Some(DenialCode::InvalidSchema),
                    detail: format!("Malformed capability URI: {}", cap),
                };
            }
        }

        let act_tier = match act.action_type.as_str() {
            "SYS_TOGGLE" | "SCROLL" | "READ_SCREEN" => ActionTier::Green,
            "OPEN_APP" | "CLICK_NODE" | "INPUT_TEXT" | "PLACE_CALL" | "SEND_MESSAGE" => ActionTier::Amber,
            "PAYMENT_INTENT" => ActionTier::Red,
            _ => {
                return ValidationOutcome {
                    decision: GatewayDecision::Deny,
                    computed_tier: None,
                    reason: Some(DenialCode::InvalidSchema),
                    detail: format!("Unsupported action_type: {}", act.action_type),
                };
            }
        };

        if act_tier > max_tier {
            max_tier = act_tier;
        }

        // Taint analysis: hostile taint invariant check
        let is_untrusted = act.data_taint_labels.iter().any(|l| {
            l == "UNTRUSTED_SCREEN_TEXT" || l == "UNTRUSTED_WEB_CONTENT" || l == "UNTRUSTED_SMS"
        });

        if is_untrusted {
            // Under Principle P3: untrusted data cannot drive RED actions or external messaging
            if act_tier == ActionTier::Red || act.action_type == "SEND_MESSAGE" {
                return ValidationOutcome {
                    decision: GatewayDecision::DenyAndAudit,
                    computed_tier: Some(act_tier),
                    reason: Some(DenialCode::TaintedInput),
                    detail: "Hostile taint violation: Untrusted text targeted at RED/telephony capability".into(),
                };
            }
        }
    }

    // 3. Formulate final decision
    let decision = match max_tier {
        ActionTier::Green => GatewayDecision::Allow,
        ActionTier::Amber => GatewayDecision::AllowWithConfirmation,
        ActionTier::Red => GatewayDecision::AllowWithConfirmation, // In production, requires biometric proof
    };

    ValidationOutcome {
        decision,
        computed_tier: Some(max_tier),
        reason: None,
        detail: "Intent validation successful".into(),
    }
}
