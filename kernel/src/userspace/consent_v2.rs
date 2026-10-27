//! Kernel Consent Broker v2 Verification Harness (SP-44)
//!
//! Constitutional References:
//! - Bible §6.3, §8.12: Consent UX + broker v2 (plain-language narratives, AIS proposals, decision audit trail)
//! - REQ-UX-301: Plain-language narratives for consent UX
//! - INV-CONSENT-001: Plain-language narrative generation
//! - INV-CONSENT-002: Immutable audit binding for decisions
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

use crate::userspace::intent_broker::{KernelActionCategory, KernelTypedProposal};
use crate::userspace::logd::{KernelLogBus, KernelLogLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelConsentRiskTier {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelUserConsentDecision {
    Pending,
    GrantOnce,
    GrantAlways,
    DenyOnce,
    DenyAlways,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelConsentCard {
    pub card_id: u32,
    pub proposal_id: u32,
    pub category: KernelActionCategory,
    pub risk_tier: KernelConsentRiskTier,
    pub title: &'static str,
    pub narrative: &'static str,
    pub resource_summary: &'static str,
    pub decision: KernelUserConsentDecision,
}

pub const MAX_PENDING_CONSENT_CARDS: usize = 8;

pub struct KernelConsentBrokerV2 {
    cards: [Option<KernelConsentCard>; MAX_PENDING_CONSENT_CARDS],
    count: usize,
    next_card_id: u32,
}

impl KernelConsentBrokerV2 {
    pub const fn new() -> Self {
        Self {
            cards: [None; MAX_PENDING_CONSENT_CARDS],
            count: 0,
            next_card_id: 5001,
        }
    }

    pub fn generate_narrative(category: KernelActionCategory, action_code: u32) -> (&'static str, &'static str, KernelConsentRiskTier) {
        match category {
            KernelActionCategory::FileOperation => {
                if action_code == 0x0002 {
                    ("Write File Request", "An assistant would like to write data to storage. This will modify file contents.", KernelConsentRiskTier::High)
                } else {
                    ("Read File Request", "An assistant would like to read a document from your files.", KernelConsentRiskTier::Medium)
                }
            }
            KernelActionCategory::NetworkTransfer => {
                ("Network Request", "An assistant requests outbound network communication to fetch information.", KernelConsentRiskTier::High)
            }
            KernelActionCategory::SystemSetting => {
                ("System Setting Modification", "An assistant is requesting permission to alter system preferences or security policies.", KernelConsentRiskTier::Critical)
            }
            KernelActionCategory::DeviceControl => {
                ("Device Control", "An assistant would like to interact with a hardware device.", KernelConsentRiskTier::Medium)
            }
            KernelActionCategory::Notification => {
                ("Post Notification", "An assistant requests displaying a user notification.", KernelConsentRiskTier::Low)
            }
        }
    }

    pub fn create_card_for_proposal(
        &mut self,
        proposal: &KernelTypedProposal,
        resource_summary: &'static str,
    ) -> Result<u32, ()> {
        if self.count >= MAX_PENDING_CONSENT_CARDS {
            return Err(());
        }

        let (title, narrative, risk_tier) = Self::generate_narrative(proposal.category, proposal.action_code);
        let id = self.next_card_id;
        self.next_card_id += 1;

        let card = KernelConsentCard {
            card_id: id,
            proposal_id: proposal.proposal_id,
            category: proposal.category,
            risk_tier,
            title,
            narrative,
            resource_summary,
            decision: KernelUserConsentDecision::Pending,
        };

        self.cards[self.count] = Some(card);
        self.count += 1;
        Ok(id)
    }

    pub fn get_card(&self, card_id: u32) -> Option<&KernelConsentCard> {
        for i in 0..self.count {
            if let Some(ref c) = self.cards[i] {
                if c.card_id == card_id {
                    return Some(c);
                }
            }
        }
        None
    }

    pub fn record_decision(
        &mut self,
        card_id: u32,
        decision: KernelUserConsentDecision,
        log_bus: &mut KernelLogBus,
    ) -> Result<KernelConsentCard, ()> {
        for i in 0..self.count {
            if let Some(ref mut c) = self.cards[i] {
                if c.card_id == card_id {
                    c.decision = decision;
                    // Immutable audit trail logging (INV-CONSENT-002)
                    let msg = match decision {
                        KernelUserConsentDecision::GrantOnce => "User granted single execution",
                        KernelUserConsentDecision::GrantAlways => "User granted permanent execution",
                        KernelUserConsentDecision::DenyOnce => "User denied single execution",
                        KernelUserConsentDecision::DenyAlways => "User denied and blacklisted execution",
                        KernelUserConsentDecision::Pending => "Consent decision pending",
                    };
                    log_bus.ingest(1000, KernelLogLevel::SecurityAudit, 7, card_id, msg);
                    return Ok(*c);
                }
            }
        }
        Err(())
    }
}
