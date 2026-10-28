//! Messages App IV: Local Search & Offline Sync (SP-138, REQ-APP-MSG-008, REQ-APP-MSG-009)
//!
//! Bible §8.18.3, §9.9.3, REQ-APP-MSG-008, REQ-APP-MSG-009
//! Invariants:
//! - INV-MSG-SEARCH-001: Strict privacy exclusion: Banking OTPs, PINs, passwords, and sensitive financial
//!                      tokens are strictly excluded from the search index to eliminate side-channel leaks.
//! - INV-MSG-SEARCH-002: AIS Proposal Card routing: AIS assistant voice and intent message generation
//!                      requires explicit user review and tap before dispatch; silent sending is prohibited.
//! - INV-MSG-SEARCH-003: Offline sync reconciliation guarantees zero message duplication upon cellular
//!                      network transition from offline to online.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

use crate::messages_core::{MAX_ADDRESS_LEN, MAX_SMS_BODY_LEN};

/// Capability required to execute local full-text search across stored messages (REQ-APP-MSG-008).
pub const CAP_APP_MSG_SEARCH: u64 = 0x1000_0000_0000;
/// Capability required for AIS to submit message proposal cards for user review (REQ-APP-MSG-009).
pub const CAP_AIS_MSG_PROPOSAL: u64 = 0x2000_0000_0000;

/// Maximum length of a search result preview snippet.
pub const MAX_SEARCH_SNIPPET_LEN: usize = 128;
/// Maximum number of search results returned per query.
pub const MAX_SEARCH_RESULTS: usize = 16;
/// Maximum number of indexed messages in the local search catalog.
pub const MAX_INDEXED_MESSAGES: usize = 32;
/// Maximum number of active AIS proposal cards.
pub const MAX_PROPOSAL_CARDS: usize = 8;
/// Maximum capacity of the offline outgoing message queue.
pub const MAX_OFFLINE_QUEUE_ENTRIES: usize = 16;
/// Search latency SLA budget in microseconds (32ms = 32,000 µs, REQ-APP-MSG-008).
pub const SEARCH_LATENCY_BUDGET_US: u32 = 32_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagesSearchError {
    MissingSearchCapability,
    MissingProposalCapability,
    SilentSendProhibited,
    ProposalNotFound,
    ProposalAlreadyProcessed,
    IndexStoreFull,
    OfflineQueueFull,
    SensitiveMessageExcluded,
    EmptyQuery,
}

/// State of an AIS Proposal Card (REQ-APP-MSG-009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalCardState {
    PendingUserReview,
    UserApproved,
    UserRejected,
    Dispatched,
}

/// Single search result entry representing a matching message.
#[derive(Debug, Clone, Copy)]
pub struct SearchResultEntry {
    pub message_id: u32,
    pub thread_id: u32,
    pub sender: [u8; MAX_ADDRESS_LEN],
    pub sender_len: usize,
    pub snippet: [u8; MAX_SEARCH_SNIPPET_LEN],
    pub snippet_len: usize,
    pub timestamp_sec: u64,
    pub match_score: u32,
}

impl SearchResultEntry {
    pub const fn empty() -> Self {
        Self {
            message_id: 0,
            thread_id: 0,
            sender: [0u8; MAX_ADDRESS_LEN],
            sender_len: 0,
            snippet: [0u8; MAX_SEARCH_SNIPPET_LEN],
            snippet_len: 0,
            timestamp_sec: 0,
            match_score: 0,
        }
    }

    pub fn sender_as_str(&self) -> &str {
        core::str::from_utf8(&self.sender[..self.sender_len]).unwrap_or("")
    }

    pub fn snippet_as_str(&self) -> &str {
        core::str::from_utf8(&self.snippet[..self.snippet_len]).unwrap_or("")
    }
}

/// Search result set containing results and telemetry (REQ-APP-MSG-008).
#[derive(Debug, Clone, Copy)]
pub struct SearchResultSet {
    pub results: [SearchResultEntry; MAX_SEARCH_RESULTS],
    pub count: usize,
    pub latency_us: u32,
    pub within_sla: bool,
}

impl SearchResultSet {
    pub const fn empty() -> Self {
        Self {
            results: [SearchResultEntry::empty(); MAX_SEARCH_RESULTS],
            count: 0,
            latency_us: 0,
            within_sla: true,
        }
    }
}

/// Indexed message entry in the local privacy-preserving search database.
#[derive(Debug, Clone, Copy)]
pub struct IndexedMessage {
    pub message_id: u32,
    pub thread_id: u32,
    pub sender: [u8; MAX_ADDRESS_LEN],
    pub sender_len: usize,
    pub body: [u8; MAX_SMS_BODY_LEN],
    pub body_len: usize,
    pub timestamp_sec: u64,
    pub is_sensitive_otp: bool,
}

impl IndexedMessage {
    pub const fn empty() -> Self {
        Self {
            message_id: 0,
            thread_id: 0,
            sender: [0u8; MAX_ADDRESS_LEN],
            sender_len: 0,
            body: [0u8; MAX_SMS_BODY_LEN],
            body_len: 0,
            timestamp_sec: 0,
            is_sensitive_otp: false,
        }
    }
}

/// AIS Message Proposal Card (REQ-APP-MSG-009, INV-MSG-SEARCH-002).
#[derive(Debug, Clone, Copy)]
pub struct MessageProposalCard {
    pub proposal_id: u32,
    pub recipient: [u8; MAX_ADDRESS_LEN],
    pub recipient_len: usize,
    pub proposed_body: [u8; MAX_SMS_BODY_LEN],
    pub body_len: usize,
    pub ais_intent_id: u32,
    pub confidence_score_ppm: u32,
    pub state: ProposalCardState,
    pub created_timestamp_sec: u64,
}

impl MessageProposalCard {
    pub const fn empty() -> Self {
        Self {
            proposal_id: 0,
            recipient: [0u8; MAX_ADDRESS_LEN],
            recipient_len: 0,
            proposed_body: [0u8; MAX_SMS_BODY_LEN],
            body_len: 0,
            ais_intent_id: 0,
            confidence_score_ppm: 0,
            state: ProposalCardState::PendingUserReview,
            created_timestamp_sec: 0,
        }
    }

    pub fn recipient_as_str(&self) -> &str {
        core::str::from_utf8(&self.recipient[..self.recipient_len]).unwrap_or("")
    }

    pub fn body_as_str(&self) -> &str {
        core::str::from_utf8(&self.proposed_body[..self.body_len]).unwrap_or("")
    }
}

/// Offline queued outgoing message item (INV-MSG-SEARCH-003).
#[derive(Debug, Clone, Copy)]
pub struct OfflineQueueItem {
    pub queue_id: u32,
    pub recipient: [u8; MAX_ADDRESS_LEN],
    pub recipient_len: usize,
    pub body: [u8; MAX_SMS_BODY_LEN],
    pub body_len: usize,
    pub enqueued_timestamp_sec: u64,
    pub dispatched_timestamp_sec: u64,
    pub delivered: bool,
    pub retry_count: u32,
}

impl OfflineQueueItem {
    pub const fn empty() -> Self {
        Self {
            queue_id: 0,
            recipient: [0u8; MAX_ADDRESS_LEN],
            recipient_len: 0,
            body: [0u8; MAX_SMS_BODY_LEN],
            body_len: 0,
            enqueued_timestamp_sec: 0,
            dispatched_timestamp_sec: 0,
            delivered: false,
            retry_count: 0,
        }
    }
}

/// Messages Search & Offline Sync Management Engine.
pub struct MessagesSearchEngine {
    index: [IndexedMessage; MAX_INDEXED_MESSAGES],
    index_count: usize,
    proposals: [MessageProposalCard; MAX_PROPOSAL_CARDS],
    proposal_count: usize,
    offline_queue: [OfflineQueueItem; MAX_OFFLINE_QUEUE_ENTRIES],
    queue_count: usize,
    next_proposal_id: u32,
    next_queue_id: u32,
    is_network_online: bool,
}

impl MessagesSearchEngine {
    pub const fn new() -> Self {
        Self {
            index: [IndexedMessage::empty(); MAX_INDEXED_MESSAGES],
            index_count: 0,
            proposals: [MessageProposalCard::empty(); MAX_PROPOSAL_CARDS],
            proposal_count: 0,
            offline_queue: [OfflineQueueItem::empty(); MAX_OFFLINE_QUEUE_ENTRIES],
            queue_count: 0,
            next_proposal_id: 1,
            next_queue_id: 1,
            is_network_online: true,
        }
    }

    pub fn index_count(&self) -> usize {
        self.index_count
    }

    pub fn proposal_count(&self) -> usize {
        self.proposal_count
    }

    pub fn queue_count(&self) -> usize {
        self.queue_count
    }

    pub fn is_network_online(&self) -> bool {
        self.is_network_online
    }

    pub fn set_network_status(&mut self, online: bool) {
        self.is_network_online = online;
    }

    /// Check whether a message body contains banking OTPs, PINs, or sensitive financial tokens (INV-MSG-SEARCH-001).
    pub fn is_sensitive_otp(body: &[u8]) -> bool {
        let sensitive_keywords: [&[u8]; 7] = [
            b"otp",
            b"code",
            b"verification",
            b"pin",
            b"password",
            b"cvv",
            b"one-time",
        ];

        for &kw in &sensitive_keywords {
            if contains_case_insensitive(body, kw) {
                return true;
            }
        }

        // Check for isolated 4-8 digit numeric codes
        let mut digit_run = 0;
        for &b in body {
            if b >= b'0' && b <= b'9' {
                digit_run += 1;
            } else {
                if digit_run >= 4 && digit_run <= 8 {
                    return true;
                }
                digit_run = 0;
            }
        }
        if digit_run >= 4 && digit_run <= 8 {
            return true;
        }

        false
    }

    /// Index a message into the local search database (INV-MSG-SEARCH-001).
    /// Sensitive OTPs and passwords are strictly excluded from indexing to safeguard privacy.
    pub fn index_message(
        &mut self,
        message_id: u32,
        thread_id: u32,
        sender: &str,
        body: &str,
        timestamp_sec: u64,
    ) -> Result<(), MessagesSearchError> {
        let body_bytes = body.as_bytes();
        let sender_bytes = sender.as_bytes();

        // INV-MSG-SEARCH-001: Strict privacy exclusion
        if Self::is_sensitive_otp(body_bytes) {
            return Err(MessagesSearchError::SensitiveMessageExcluded);
        }

        if self.index_count >= MAX_INDEXED_MESSAGES {
            return Err(MessagesSearchError::IndexStoreFull);
        }

        let mut entry = IndexedMessage::empty();
        entry.message_id = message_id;
        entry.thread_id = thread_id;

        let slen = sender_bytes.len().min(MAX_ADDRESS_LEN);
        entry.sender[..slen].copy_from_slice(&sender_bytes[..slen]);
        entry.sender_len = slen;

        let blen = body_bytes.len().min(MAX_SMS_BODY_LEN);
        entry.body[..blen].copy_from_slice(&body_bytes[..blen]);
        entry.body_len = blen;

        entry.timestamp_sec = timestamp_sec;
        entry.is_sensitive_otp = false;

        self.index[self.index_count] = entry;
        self.index_count += 1;
        Ok(())
    }

    /// Search indexed messages across sender and body (REQ-APP-MSG-008).
    /// Enforces capability check and validates sub-32ms latency SLA.
    pub fn search(
        &self,
        capabilities: u64,
        query: &str,
        simulated_latency_us: u32,
    ) -> Result<SearchResultSet, MessagesSearchError> {
        if capabilities & CAP_APP_MSG_SEARCH == 0 {
            return Err(MessagesSearchError::MissingSearchCapability);
        }

        let q_bytes = query.as_bytes();
        if q_bytes.is_empty() {
            return Err(MessagesSearchError::EmptyQuery);
        }

        let mut set = SearchResultSet::empty();
        set.latency_us = simulated_latency_us;
        set.within_sla = simulated_latency_us <= SEARCH_LATENCY_BUDGET_US;

        for i in 0..self.index_count {
            let msg = &self.index[i];
            let mut match_score = 0;

            let sender_match = contains_case_insensitive(&msg.sender[..msg.sender_len], q_bytes);
            let body_match = contains_case_insensitive(&msg.body[..msg.body_len], q_bytes);

            if sender_match {
                match_score += 50;
            }
            if body_match {
                match_score += 30;
            }

            if match_score > 0 && set.count < MAX_SEARCH_RESULTS {
                let mut res = SearchResultEntry::empty();
                res.message_id = msg.message_id;
                res.thread_id = msg.thread_id;
                res.sender[..msg.sender_len].copy_from_slice(&msg.sender[..msg.sender_len]);
                res.sender_len = msg.sender_len;

                let snip_len = msg.body_len.min(MAX_SEARCH_SNIPPET_LEN);
                res.snippet[..snip_len].copy_from_slice(&msg.body[..snip_len]);
                res.snippet_len = snip_len;

                res.timestamp_sec = msg.timestamp_sec;
                res.match_score = match_score;

                set.results[set.count] = res;
                set.count += 1;
            }
        }

        Ok(set)
    }

    /// Submit an AIS message proposal card for user review (REQ-APP-MSG-009).
    /// Requires CAP_AIS_MSG_PROPOSAL.
    pub fn propose_message(
        &mut self,
        capabilities: u64,
        ais_intent_id: u32,
        recipient: &str,
        proposed_body: &str,
        confidence_score_ppm: u32,
        timestamp_sec: u64,
    ) -> Result<u32, MessagesSearchError> {
        if capabilities & CAP_AIS_MSG_PROPOSAL == 0 {
            return Err(MessagesSearchError::MissingProposalCapability);
        }

        if self.proposal_count >= MAX_PROPOSAL_CARDS {
            return Err(MessagesSearchError::IndexStoreFull);
        }

        let prop_id = self.next_proposal_id;
        self.next_proposal_id += 1;

        let mut card = MessageProposalCard::empty();
        card.proposal_id = prop_id;
        card.ais_intent_id = ais_intent_id;
        card.confidence_score_ppm = confidence_score_ppm;
        card.state = ProposalCardState::PendingUserReview;
        card.created_timestamp_sec = timestamp_sec;

        let r_bytes = recipient.as_bytes();
        let r_len = r_bytes.len().min(MAX_ADDRESS_LEN);
        card.recipient[..r_len].copy_from_slice(&r_bytes[..r_len]);
        card.recipient_len = r_len;

        let b_bytes = proposed_body.as_bytes();
        let b_len = b_bytes.len().min(MAX_SMS_BODY_LEN);
        card.proposed_body[..b_len].copy_from_slice(&b_bytes[..b_len]);
        card.body_len = b_len;

        self.proposals[self.proposal_count] = card;
        self.proposal_count += 1;

        Ok(prop_id)
    }

    /// Explicit user approval and dispatch of an AIS Proposal Card (REQ-APP-MSG-009, INV-MSG-SEARCH-002).
    /// Calling this with user_confirmed = false or attempting silent dispatch will fail with SilentSendProhibited.
    pub fn confirm_and_dispatch_proposal(
        &mut self,
        proposal_id: u32,
        user_confirmed: bool,
    ) -> Result<(), MessagesSearchError> {
        // INV-MSG-SEARCH-002: Silent send prohibited
        if !user_confirmed {
            return Err(MessagesSearchError::SilentSendProhibited);
        }

        let idx = self
            .proposals
            .iter()
            .position(|p| p.proposal_id == proposal_id)
            .ok_or(MessagesSearchError::ProposalNotFound)?;

        if self.proposals[idx].state != ProposalCardState::PendingUserReview {
            return Err(MessagesSearchError::ProposalAlreadyProcessed);
        }

        self.proposals[idx].state = ProposalCardState::Dispatched;
        Ok(())
    }

    /// User explicitly rejects an AIS proposal card (REQ-APP-MSG-009).
    pub fn reject_proposal(&mut self, proposal_id: u32) -> Result<(), MessagesSearchError> {
        let idx = self
            .proposals
            .iter()
            .position(|p| p.proposal_id == proposal_id)
            .ok_or(MessagesSearchError::ProposalNotFound)?;

        if self.proposals[idx].state != ProposalCardState::PendingUserReview {
            return Err(MessagesSearchError::ProposalAlreadyProcessed);
        }

        self.proposals[idx].state = ProposalCardState::UserRejected;
        Ok(())
    }

    /// Enqueue an outgoing message during offline periods (INV-MSG-SEARCH-003).
    pub fn enqueue_offline_message(
        &mut self,
        recipient: &str,
        body: &str,
        timestamp_sec: u64,
    ) -> Result<u32, MessagesSearchError> {
        if self.queue_count >= MAX_OFFLINE_QUEUE_ENTRIES {
            return Err(MessagesSearchError::OfflineQueueFull);
        }

        let q_id = self.next_queue_id;
        self.next_queue_id += 1;

        let mut item = OfflineQueueItem::empty();
        item.queue_id = q_id;
        item.enqueued_timestamp_sec = timestamp_sec;
        item.dispatched_timestamp_sec = 0;
        item.delivered = false;
        item.retry_count = 0;

        let r_bytes = recipient.as_bytes();
        let r_len = r_bytes.len().min(MAX_ADDRESS_LEN);
        item.recipient[..r_len].copy_from_slice(&r_bytes[..r_len]);
        item.recipient_len = r_len;

        let b_bytes = body.as_bytes();
        let b_len = b_bytes.len().min(MAX_SMS_BODY_LEN);
        item.body[..b_len].copy_from_slice(&b_bytes[..b_len]);
        item.body_len = b_len;

        self.offline_queue[self.queue_count] = item;
        self.queue_count += 1;

        Ok(q_id)
    }

    /// Reconcile offline queue when cellular connectivity transitions to online (INV-MSG-SEARCH-003).
    /// Dispatches all undelivered items in FIFO order without duplication.
    /// Returns the number of successfully reconciled and dispatched messages.
    pub fn reconcile_offline_queue(&mut self, current_time_sec: u64) -> usize {
        self.is_network_online = true;
        let mut dispatched_count = 0;

        for i in 0..self.queue_count {
            let item = &mut self.offline_queue[i];
            if !item.delivered {
                item.delivered = true;
                item.dispatched_timestamp_sec = current_time_sec;
                item.retry_count += 1;
                dispatched_count += 1;
            }
        }

        dispatched_count
    }

    pub fn get_proposal(&self, proposal_id: u32) -> Option<&MessageProposalCard> {
        self.proposals.iter().find(|p| p.proposal_id == proposal_id)
    }

    pub fn get_queue_item(&self, queue_id: u32) -> Option<&OfflineQueueItem> {
        self.offline_queue.iter().find(|q| q.queue_id == queue_id)
    }
}

/// Case-insensitive ASCII substring search helper.
fn contains_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }

    for i in 0..=(haystack.len() - needle.len()) {
        let mut matches = true;
        for j in 0..needle.len() {
            let h = to_ascii_lower(haystack[i + j]);
            let n = to_ascii_lower(needle[j]);
            if h != n {
                matches = false;
                break;
            }
        }
        if matches {
            return true;
        }
    }
    false
}

#[inline(always)]
fn to_ascii_lower(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otp_exclusion_from_search_index() {
        let mut engine = MessagesSearchEngine::new();

        // 1. Regular message should index successfully
        let res = engine.index_message(
            1,
            100,
            "+919876543210",
            "Hey, are we still meeting for lunch at 1pm?",
            1700000000,
        );
        assert!(res.is_ok());
        assert_eq!(engine.index_count(), 1);

        // 2. Sensitive banking OTP must be rejected (INV-MSG-SEARCH-001)
        let res_otp = engine.index_message(
            2,
            101,
            "SBI-ALERT",
            "Your SBI net banking OTP is 492015. Valid for 5 mins. Do not share.",
            1700000010,
        );
        assert_eq!(res_otp, Err(MessagesSearchError::SensitiveMessageExcluded));
        assert_eq!(engine.index_count(), 1); // Not indexed!

        // 3. Password reset message must be rejected
        let res_pwd = engine.index_message(
            3,
            102,
            "AUTH-MSG",
            "Your temporary password is securePass123. Reset immediately.",
            1700000020,
        );
        assert_eq!(res_pwd, Err(MessagesSearchError::SensitiveMessageExcluded));
        assert_eq!(engine.index_count(), 1);

        // 4. Verify search finds the regular message and cannot find OTP
        let search_res = engine.search(CAP_APP_MSG_SEARCH, "lunch", 1_200).unwrap();
        assert_eq!(search_res.count, 1);
        assert_eq!(search_res.results[0].message_id, 1);
        assert!(search_res.within_sla);

        // Search for OTP keywords returns 0 results
        let search_otp = engine.search(CAP_APP_MSG_SEARCH, "OTP", 800).unwrap();
        assert_eq!(search_otp.count, 0);
    }

    #[test]
    fn test_search_requires_capability_and_verifies_sla() {
        let mut engine = MessagesSearchEngine::new();
        engine
            .index_message(
                1,
                10,
                "+918888888888",
                "Flight to Delhi confirmed for tomorrow morning",
                1700001000,
            )
            .unwrap();

        // Without capability: must fail
        let unauthorized = engine.search(0, "Delhi", 500);
        assert_eq!(unauthorized.err(), Some(MessagesSearchError::MissingSearchCapability));

        // With capability: must succeed
        let authorized = engine.search(CAP_APP_MSG_SEARCH, "Delhi", 15_000).unwrap();
        assert_eq!(authorized.count, 1);
        assert!(authorized.within_sla);

        // Latency exceeding 32ms budget (REQ-APP-MSG-008)
        let over_budget = engine.search(CAP_APP_MSG_SEARCH, "Delhi", 35_000).unwrap();
        assert_eq!(over_budget.count, 1);
        assert!(!over_budget.within_sla);
    }

    #[test]
    fn test_ais_proposal_card_workflow_and_silent_send_prevention() {
        let mut engine = MessagesSearchEngine::new();

        // 1. AIS proposes a message without capability: fails
        let unauth = engine.propose_message(
            0,
            101,
            "+919876543210",
            "I am on my way home now.",
            990_000,
            1700002000,
        );
        assert_eq!(unauth.err(), Some(MessagesSearchError::MissingProposalCapability));

        // 2. AIS proposes message with capability
        let prop_id = engine
            .propose_message(
                CAP_AIS_MSG_PROPOSAL,
                101,
                "+919876543210",
                "I am on my way home now.",
                990_000,
                1700002000,
            )
            .unwrap();

        let card = engine.get_proposal(prop_id).unwrap();
        assert_eq!(card.state, ProposalCardState::PendingUserReview);
        assert_eq!(card.recipient_as_str(), "+919876543210");

        // 3. Silent dispatch attempt (user_confirmed = false): must be blocked (INV-MSG-SEARCH-002)
        let silent_err = engine.confirm_and_dispatch_proposal(prop_id, false);
        assert_eq!(silent_err, Err(MessagesSearchError::SilentSendProhibited));

        // 4. Explicit user confirmation: succeeds
        let dispatch_ok = engine.confirm_and_dispatch_proposal(prop_id, true);
        assert!(dispatch_ok.is_ok());

        let dispatched_card = engine.get_proposal(prop_id).unwrap();
        assert_eq!(dispatched_card.state, ProposalCardState::Dispatched);

        // 5. Subsequent attempt fails with ProposalAlreadyProcessed
        let re_dispatch = engine.confirm_and_dispatch_proposal(prop_id, true);
        assert_eq!(re_dispatch, Err(MessagesSearchError::ProposalAlreadyProcessed));
    }

    #[test]
    fn test_offline_queue_reconciliation_zero_duplication() {
        let mut engine = MessagesSearchEngine::new();
        engine.set_network_status(false);

        // Queue messages while offline
        let q1 = engine.enqueue_offline_message("+919999999991", "Offline msg 1", 1700003000).unwrap();
        let q2 = engine.enqueue_offline_message("+919999999992", "Offline msg 2", 1700003010).unwrap();
        assert_eq!(engine.queue_count(), 2);

        let item1 = engine.get_queue_item(q1).unwrap();
        assert!(!item1.delivered);
        assert_eq!(item1.dispatched_timestamp_sec, 0);

        // Cellular network reconnects: reconcile
        let dispatched = engine.reconcile_offline_queue(1700003500);
        assert_eq!(dispatched, 2);
        assert!(engine.is_network_online());

        let item1_reconciled = engine.get_queue_item(q1).unwrap();
        assert!(item1_reconciled.delivered);
        assert_eq!(item1_reconciled.dispatched_timestamp_sec, 1700003500);

        let item2_reconciled = engine.get_queue_item(q2).unwrap();
        assert!(item2_reconciled.delivered);
        assert_eq!(item2_reconciled.dispatched_timestamp_sec, 1700003500);

        // Reconcile again: no duplicate dispatches
        let duplicate_run = engine.reconcile_offline_queue(1700003600);
        assert_eq!(duplicate_run, 0);
    }
}
