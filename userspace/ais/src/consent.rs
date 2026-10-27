//! Ind OS Consent Broker v2 Subsystem (`indus-ais::consent`)
//!
//! Standards:
//! - REQ-UX-301: Plain-language narratives for consent UX
//! - INV-CONSENT-001: Plain-language narrative generation
//! - INV-CONSENT-002: Immutable audit binding for decisions
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::schema::ActionCategory;
use crate::intent::TypedProposal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentRiskTier {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserConsentDecision {
    Pending,
    GrantOnce,
    GrantAlways,
    DenyOnce,
    DenyAlways,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsentCard {
    pub card_id: u32,
    pub proposal_id: u32,
    pub category: ActionCategory,
    pub risk_tier: ConsentRiskTier,
    pub title: &'static str,
    pub narrative: &'static str,
    pub resource_summary: &'static str,
    pub decision: UserConsentDecision,
}

pub const MAX_PENDING_CONSENT_CARDS: usize = 8;

pub struct ConsentBrokerV2 {
    cards: [Option<ConsentCard>; MAX_PENDING_CONSENT_CARDS],
    count: usize,
    next_card_id: u32,
}

impl ConsentBrokerV2 {
    pub const fn new() -> Self {
        Self {
            cards: [None; MAX_PENDING_CONSENT_CARDS],
            count: 0,
            next_card_id: 5001,
        }
    }

    pub fn generate_narrative(category: ActionCategory, action_code: u32) -> (&'static str, &'static str, ConsentRiskTier) {
        match category {
            ActionCategory::FileOperation => {
                if action_code == 0x0002 {
                    ("Write File Request", "An assistant would like to write data to storage. This will modify file contents.", ConsentRiskTier::High)
                } else {
                    ("Read File Request", "An assistant would like to read a document from your files.", ConsentRiskTier::Medium)
                }
            }
            ActionCategory::NetworkTransfer => {
                ("Network Request", "An assistant requests outbound network communication to fetch information.", ConsentRiskTier::High)
            }
            ActionCategory::SystemSetting => {
                ("System Setting Modification", "An assistant is requesting permission to alter system preferences or security policies.", ConsentRiskTier::Critical)
            }
            ActionCategory::DeviceControl => {
                ("Device Control", "An assistant would like to interact with a hardware device.", ConsentRiskTier::Medium)
            }
            ActionCategory::Notification => {
                ("Post Notification", "An assistant requests displaying a user notification.", ConsentRiskTier::Low)
            }
        }
    }

    pub fn create_card_for_proposal(
        &mut self,
        proposal: &TypedProposal,
        resource_summary: &'static str,
    ) -> Result<u32, ()> {
        if self.count >= MAX_PENDING_CONSENT_CARDS {
            return Err(());
        }

        let (title, narrative, risk_tier) = Self::generate_narrative(proposal.category, proposal.action_code);
        let id = self.next_card_id;
        self.next_card_id += 1;

        let card = ConsentCard {
            card_id: id,
            proposal_id: proposal.proposal_id,
            category: proposal.category,
            risk_tier,
            title,
            narrative,
            resource_summary,
            decision: UserConsentDecision::Pending,
        };

        self.cards[self.count] = Some(card);
        self.count += 1;
        Ok(id)
    }

    pub fn get_card(&self, card_id: u32) -> Option<&ConsentCard> {
        for i in 0..self.count {
            if let Some(ref c) = self.cards[i] {
                if c.card_id == card_id {
                    return Some(c);
                }
            }
        }
        None
    }

    pub fn record_decision(&mut self, card_id: u32, decision: UserConsentDecision) -> Result<ConsentCard, ()> {
        for i in 0..self.count {
            if let Some(ref mut c) = self.cards[i] {
                if c.card_id == card_id {
                    c.decision = decision;
                    return Ok(*c);
                }
            }
        }
        Err(())
    }
}
