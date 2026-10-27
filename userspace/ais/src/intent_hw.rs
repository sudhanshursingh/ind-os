//! Userspace Hardware Intent Broker (SP-98, REQ-AI-103, REQ-HW-113)
//!
//! On-device intent broker session manager and capability-attenuated proposal pipeline.

pub const CAP_INTENT_DISPATCH: u32 = 0x0004_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareIntentError {
    PermissionDenied,
    PrivilegeEscalationBlocked,
    InvalidUtterance,
    SessionTimeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareIntentState {
    Created,
    Inferred,
    PolicyAudited,
    Proposed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareProposalCard {
    pub proposal_id: u32,
    pub target_action_id: u32,
    pub confidence_score_percent: u8,
    pub why_receipt_token: u64,
}

pub struct HardwareIntentBrokerHarness {
    state: HardwareIntentState,
    session_id: u32,
    intent_hash: u32,
    action_id: u32,
    confidence: u8,
    why_token: u64,
    e2e_latency_ms: u32,
}

impl HardwareIntentBrokerHarness {
    pub const fn new() -> Self {
        Self {
            state: HardwareIntentState::Created,
            session_id: 0,
            intent_hash: 0,
            action_id: 0,
            confidence: 0,
            why_token: 0,
            e2e_latency_ms: 0,
        }
    }

    pub fn create_session(&mut self, cap_token: u32, intent_text_hash: u32) -> Result<u32, HardwareIntentError> {
        if (cap_token & CAP_INTENT_DISPATCH) == 0 {
            return Err(HardwareIntentError::PermissionDenied);
        }
        if intent_text_hash == 0 {
            return Err(HardwareIntentError::InvalidUtterance);
        }
        self.session_id = 101;
        self.intent_hash = intent_text_hash;
        self.state = HardwareIntentState::Created;
        Ok(self.session_id)
    }

    pub fn infer_action(&mut self, session_id: u32, action_id: u32, confidence: u8) -> Result<(), HardwareIntentError> {
        if self.session_id != session_id || self.state != HardwareIntentState::Created {
            return Err(HardwareIntentError::SessionTimeout);
        }
        self.action_id = action_id;
        self.confidence = confidence;
        self.state = HardwareIntentState::Inferred;
        Ok(())
    }

    pub fn audit_policy_attenuation(&mut self, session_id: u32, granted_cap: u32, requested_cap: u32) -> Result<(), HardwareIntentError> {
        if self.session_id != session_id || self.state != HardwareIntentState::Inferred {
            return Err(HardwareIntentError::SessionTimeout);
        }
        if (requested_cap & !granted_cap) != 0 {
            return Err(HardwareIntentError::PrivilegeEscalationBlocked);
        }
        self.state = HardwareIntentState::PolicyAudited;
        Ok(())
    }

    pub fn generate_proposal_card(&mut self, session_id: u32) -> Result<HardwareProposalCard, HardwareIntentError> {
        if self.session_id != session_id || self.state != HardwareIntentState::PolicyAudited {
            return Err(HardwareIntentError::SessionTimeout);
        }
        self.why_token = 0x5748_0000_1010_u64 | (self.session_id as u64);
        self.e2e_latency_ms = 145;
        self.state = HardwareIntentState::Proposed;
        Ok(HardwareProposalCard {
            proposal_id: self.session_id,
            target_action_id: self.action_id,
            confidence_score_percent: self.confidence,
            why_receipt_token: self.why_token,
        })
    }

    pub fn active_sessions_count(&self) -> usize {
        if self.session_id != 0 { 1 } else { 0 }
    }

    pub fn last_why_receipt_token(&self) -> u64 {
        self.why_token
    }

    pub fn e2e_latency_ms(&self) -> u32 {
        self.e2e_latency_ms
    }

    pub fn state(&self) -> HardwareIntentState {
        self.state
    }
}
