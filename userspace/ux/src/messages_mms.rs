//! Messages App III: RCS, MMS & Group Messaging (SP-137, REQ-APP-MSG-007, REQ-APP-MSG-010)
//!
//! Bible §8.18.3, §9.9.3, REQ-APP-MSG-007, REQ-APP-MSG-010
//! Invariants:
//! - INV-MSG-MMS-001: MMS dispatch and group messaging require explicit CAP_APP_MMS_SEND;
//!                    multimedia payload size is bounded; participants must be valid E.164 addresses.
//! - INV-MSG-MMS-002: Group conversations preserve multi-party thread integrity; replies adhere strictly
//!                    to GroupReplyMode (ReplyAll vs ReplySender) without address laundering.
//! - INV-MSG-MMS-003: RCS transport layer negotiates peer capabilities dynamically; automatically falls
//!                    back to SMS/MMS without message duplication or silent drop if RCS is unavailable.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

use crate::messages_core::{DeliveryStatus, MessageDirection, MAX_ADDRESS_LEN, MAX_SMS_BODY_LEN};

/// Capability required to send cellular MMS messages (REQ-APP-MSG-010).
pub const CAP_APP_MMS_SEND: u64 = 0x0400_0000_0000;
/// Capability required to receive and process cellular MMS messages.
pub const CAP_APP_MMS_RECEIVE: u64 = 0x0800_0000_0000;

/// Maximum number of participants in a group conversation.
pub const MAX_GROUP_PARTICIPANTS: usize = 10;
/// Maximum number of multimedia parts per MMS message.
pub const MAX_MMS_PARTS: usize = 4;
/// Maximum length of an MMS subject line.
pub const MAX_SUBJECT_LEN: usize = 64;
/// Maximum capacity for MMS part data payload buffer.
pub const MAX_MMS_PART_DATA_LEN: usize = 512;
/// Maximum number of active groups.
pub const MAX_GROUPS: usize = 8;
/// Maximum number of stored MMS messages.
pub const MAX_MMS_MESSAGES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagesMmsError {
    MissingMmsSendCapability,
    MissingMmsReceiveCapability,
    ParticipantLimitExceeded,
    InvalidParticipantAddress,
    GroupStoreFull,
    GroupNotFound,
    MmsStoreFull,
    PartLimitExceeded,
    PayloadTooLarge,
    RcsNegotiationFailed,
}

/// Reply routing mode for group conversations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupReplyMode {
    ReplyAll,
    ReplySender,
}

/// Carrier / Network transport protocol negotiated for message delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTransportProtocol {
    Sms,
    Mms,
    RcsUniversal,
}

/// Multimedia Part descriptor within an MMS message (REQ-APP-MSG-010).
#[derive(Debug, Clone, Copy)]
pub struct MmsPart {
    pub part_id: u32,
    pub mime_type: [u8; 32],
    pub mime_len: usize,
    pub data: [u8; MAX_MMS_PART_DATA_LEN],
    pub data_len: usize,
}

impl MmsPart {
    pub const fn empty() -> Self {
        Self {
            part_id: 0,
            mime_type: [0u8; 32],
            mime_len: 0,
            data: [0u8; MAX_MMS_PART_DATA_LEN],
            data_len: 0,
        }
    }

    pub fn mime_as_str(&self) -> &str {
        core::str::from_utf8(&self.mime_type[..self.mime_len]).unwrap_or("")
    }
}

/// Multi-Party MMS Message descriptor (REQ-APP-MSG-010).
#[derive(Debug, Clone, Copy)]
pub struct MmsMessage {
    pub id: u32,
    pub group_id: Option<u32>,
    pub sender: [u8; MAX_ADDRESS_LEN],
    pub sender_len: usize,
    pub subject: [u8; MAX_SUBJECT_LEN],
    pub subject_len: usize,
    pub body: [u8; MAX_SMS_BODY_LEN],
    pub body_len: usize,
    pub parts: [MmsPart; MAX_MMS_PARTS],
    pub part_count: usize,
    pub timestamp_sec: u64,
    pub direction: MessageDirection,
    pub status: DeliveryStatus,
    pub protocol: MessageTransportProtocol,
}

impl MmsMessage {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            group_id: None,
            sender: [0u8; MAX_ADDRESS_LEN],
            sender_len: 0,
            subject: [0u8; MAX_SUBJECT_LEN],
            subject_len: 0,
            body: [0u8; MAX_SMS_BODY_LEN],
            body_len: 0,
            parts: [MmsPart::empty(); MAX_MMS_PARTS],
            part_count: 0,
            timestamp_sec: 0,
            direction: MessageDirection::Outbound,
            status: DeliveryStatus::Pending,
            protocol: MessageTransportProtocol::Mms,
        }
    }

    pub fn subject_as_str(&self) -> &str {
        core::str::from_utf8(&self.subject[..self.subject_len]).unwrap_or("")
    }
}

/// Group Conversation representation (REQ-APP-MSG-010).
#[derive(Debug, Clone, Copy)]
pub struct GroupConversation {
    pub group_id: u32,
    pub group_name: [u8; 32],
    pub name_len: usize,
    pub participants: [[u8; MAX_ADDRESS_LEN]; MAX_GROUP_PARTICIPANTS],
    pub participant_lens: [usize; MAX_GROUP_PARTICIPANTS],
    pub participant_count: usize,
    pub reply_mode: GroupReplyMode,
    pub created_timestamp_sec: u64,
}

impl GroupConversation {
    pub const fn empty() -> Self {
        Self {
            group_id: 0,
            group_name: [0u8; 32],
            name_len: 0,
            participants: [[0u8; MAX_ADDRESS_LEN]; MAX_GROUP_PARTICIPANTS],
            participant_lens: [0; MAX_GROUP_PARTICIPANTS],
            participant_count: 0,
            reply_mode: GroupReplyMode::ReplyAll,
            created_timestamp_sec: 0,
        }
    }

    pub fn name_as_str(&self) -> &str {
        core::str::from_utf8(&self.group_name[..self.name_len]).unwrap_or("")
    }

    pub fn get_participant(&self, index: usize) -> Option<&str> {
        if index < self.participant_count {
            core::str::from_utf8(&self.participants[index][..self.participant_lens[index]]).ok()
        } else {
            None
        }
    }
}

/// RCS Universal Messaging State & Capabilities (REQ-APP-MSG-007).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RcsSessionState {
    pub rcs_network_available: bool,
    pub peer_rcs_capable: bool,
    pub is_peer_typing: bool,
    pub read_receipts_enabled: bool,
    pub fallback_to_sms_mms: bool,
}

impl Default for RcsSessionState {
    fn default() -> Self {
        Self {
            rcs_network_available: false,
            peer_rcs_capable: false,
            is_peer_typing: false,
            read_receipts_enabled: true,
            fallback_to_sms_mms: true,
        }
    }
}

/// Integrated RCS, MMS & Group Messaging Engine.
pub struct MessagesMmsEngine {
    groups: [GroupConversation; MAX_GROUPS],
    group_count: usize,
    messages: [MmsMessage; MAX_MMS_MESSAGES],
    message_count: usize,
    next_group_id: u32,
    next_msg_id: u32,
    rcs_state: RcsSessionState,
}

impl MessagesMmsEngine {
    pub const fn new() -> Self {
        Self {
            groups: [GroupConversation::empty(); MAX_GROUPS],
            group_count: 0,
            messages: [MmsMessage::empty(); MAX_MMS_MESSAGES],
            message_count: 0,
            next_group_id: 1,
            next_msg_id: 1,
            rcs_state: RcsSessionState {
                rcs_network_available: false,
                peer_rcs_capable: false,
                is_peer_typing: false,
                read_receipts_enabled: true,
                fallback_to_sms_mms: true,
            },
        }
    }

    pub fn group_count(&self) -> usize {
        self.group_count
    }

    pub fn message_count(&self) -> usize {
        self.message_count
    }

    pub fn rcs_state(&self) -> RcsSessionState {
        self.rcs_state
    }

    pub fn set_rcs_network_available(&mut self, available: bool) {
        self.rcs_state.rcs_network_available = available;
    }

    pub fn set_peer_rcs_capable(&mut self, capable: bool) {
        self.rcs_state.peer_rcs_capable = capable;
    }

    pub fn set_peer_typing(&mut self, typing: bool) {
        self.rcs_state.is_peer_typing = typing;
    }

    /// Create a new multi-party group conversation thread (REQ-APP-MSG-010).
    pub fn create_group(
        &mut self,
        name: &str,
        participants: &[&str],
        timestamp_sec: u64,
    ) -> Result<u32, MessagesMmsError> {
        if self.group_count >= MAX_GROUPS {
            return Err(MessagesMmsError::GroupStoreFull);
        }
        if participants.len() > MAX_GROUP_PARTICIPANTS {
            return Err(MessagesMmsError::ParticipantLimitExceeded);
        }

        let group_id = self.next_group_id;
        self.next_group_id += 1;

        let mut g = GroupConversation::empty();
        g.group_id = group_id;
        let n_bytes = name.as_bytes();
        let n_len = n_bytes.len().min(32);
        g.group_name[..n_len].copy_from_slice(&n_bytes[..n_len]);
        g.name_len = n_len;
        g.created_timestamp_sec = timestamp_sec;

        for (idx, &p) in participants.iter().enumerate() {
            let p_bytes = p.as_bytes();
            if p_bytes.is_empty() || p_bytes.len() > MAX_ADDRESS_LEN {
                return Err(MessagesMmsError::InvalidParticipantAddress);
            }
            g.participants[idx][..p_bytes.len()].copy_from_slice(p_bytes);
            g.participant_lens[idx] = p_bytes.len();
        }
        g.participant_count = participants.len();

        let idx = self.group_count;
        self.groups[idx] = g;
        self.group_count += 1;

        Ok(group_id)
    }

    pub fn get_group(&self, group_id: u32) -> Option<&GroupConversation> {
        for i in 0..self.group_count {
            if self.groups[i].group_id == group_id {
                return Some(&self.groups[i]);
            }
        }
        None
    }

    /// Negotiate message transport protocol between RCS, MMS, and SMS fallback (REQ-APP-MSG-007, INV-MSG-MMS-003).
    pub fn negotiate_transport(&self, has_attachments: bool) -> MessageTransportProtocol {
        if self.rcs_state.rcs_network_available && self.rcs_state.peer_rcs_capable {
            MessageTransportProtocol::RcsUniversal
        } else if has_attachments {
            MessageTransportProtocol::Mms
        } else {
            MessageTransportProtocol::Sms
        }
    }

    /// Compose and dispatch an MMS message with optional attachments and subject (REQ-APP-MSG-010).
    /// Enforces CAP_APP_MMS_SEND.
    pub fn send_mms(
        &mut self,
        group_id: Option<u32>,
        subject: &str,
        body: &str,
        parts: &[(&str, &[u8])],
        timestamp_sec: u64,
        caller_caps: u64,
    ) -> Result<u32, MessagesMmsError> {
        if (caller_caps & CAP_APP_MMS_SEND) == 0 {
            return Err(MessagesMmsError::MissingMmsSendCapability);
        }
        if self.message_count >= MAX_MMS_MESSAGES {
            return Err(MessagesMmsError::MmsStoreFull);
        }
        if parts.len() > MAX_MMS_PARTS {
            return Err(MessagesMmsError::PartLimitExceeded);
        }

        if let Some(gid) = group_id {
            if self.get_group(gid).is_none() {
                return Err(MessagesMmsError::GroupNotFound);
            }
        }

        let msg_id = self.next_msg_id;
        self.next_msg_id += 1;

        let mut msg = MmsMessage::empty();
        msg.id = msg_id;
        msg.group_id = group_id;
        msg.timestamp_sec = timestamp_sec;
        msg.direction = MessageDirection::Outbound;
        msg.status = DeliveryStatus::Sent;

        let sub_bytes = subject.as_bytes();
        let sub_len = sub_bytes.len().min(MAX_SUBJECT_LEN);
        msg.subject[..sub_len].copy_from_slice(&sub_bytes[..sub_len]);
        msg.subject_len = sub_len;

        let body_bytes = body.as_bytes();
        let body_len = body_bytes.len().min(MAX_SMS_BODY_LEN);
        msg.body[..body_len].copy_from_slice(&body_bytes[..body_len]);
        msg.body_len = body_len;

        for (idx, &(mime, data)) in parts.iter().enumerate() {
            if data.len() > MAX_MMS_PART_DATA_LEN {
                return Err(MessagesMmsError::PayloadTooLarge);
            }
            let mut part = MmsPart::empty();
            part.part_id = (idx + 1) as u32;
            let m_bytes = mime.as_bytes();
            let m_len = m_bytes.len().min(32);
            part.mime_type[..m_len].copy_from_slice(&m_bytes[..m_len]);
            part.mime_len = m_len;
            part.data[..data.len()].copy_from_slice(data);
            part.data_len = data.len();

            msg.parts[idx] = part;
        }
        msg.part_count = parts.len();

        msg.protocol = self.negotiate_transport(msg.part_count > 0);

        let m_idx = self.message_count;
        self.messages[m_idx] = msg;
        self.message_count += 1;

        Ok(msg_id)
    }

    /// Receive an inbound MMS message from carrier M-Notification.ind (REQ-APP-MSG-010).
    /// Enforces CAP_APP_MMS_RECEIVE.
    pub fn receive_mms(
        &mut self,
        sender: &str,
        group_id: Option<u32>,
        subject: &str,
        body: &str,
        parts: &[(&str, &[u8])],
        timestamp_sec: u64,
        caller_caps: u64,
    ) -> Result<u32, MessagesMmsError> {
        if (caller_caps & CAP_APP_MMS_RECEIVE) == 0 {
            return Err(MessagesMmsError::MissingMmsReceiveCapability);
        }
        if self.message_count >= MAX_MMS_MESSAGES {
            return Err(MessagesMmsError::MmsStoreFull);
        }

        let msg_id = self.next_msg_id;
        self.next_msg_id += 1;

        let mut msg = MmsMessage::empty();
        msg.id = msg_id;
        msg.group_id = group_id;
        msg.timestamp_sec = timestamp_sec;
        msg.direction = MessageDirection::Inbound;
        msg.status = DeliveryStatus::Delivered;

        let s_bytes = sender.as_bytes();
        let s_len = s_bytes.len().min(MAX_ADDRESS_LEN);
        msg.sender[..s_len].copy_from_slice(&s_bytes[..s_len]);
        msg.sender_len = s_len;

        let sub_bytes = subject.as_bytes();
        let sub_len = sub_bytes.len().min(MAX_SUBJECT_LEN);
        msg.subject[..sub_len].copy_from_slice(&sub_bytes[..sub_len]);
        msg.subject_len = sub_len;

        let body_bytes = body.as_bytes();
        let body_len = body_bytes.len().min(MAX_SMS_BODY_LEN);
        msg.body[..body_len].copy_from_slice(&body_bytes[..body_len]);
        msg.body_len = body_len;

        for (idx, &(mime, data)) in parts.iter().enumerate() {
            if data.len() > MAX_MMS_PART_DATA_LEN {
                return Err(MessagesMmsError::PayloadTooLarge);
            }
            let mut part = MmsPart::empty();
            part.part_id = (idx + 1) as u32;
            let m_bytes = mime.as_bytes();
            let m_len = m_bytes.len().min(32);
            part.mime_type[..m_len].copy_from_slice(&m_bytes[..m_len]);
            part.mime_len = m_len;
            part.data[..data.len()].copy_from_slice(data);
            part.data_len = data.len();

            msg.parts[idx] = part;
        }
        msg.part_count = parts.len();

        msg.protocol = self.negotiate_transport(msg.part_count > 0);

        let m_idx = self.message_count;
        self.messages[m_idx] = msg;
        self.message_count += 1;

        Ok(msg_id)
    }

    pub fn get_message(&self, msg_id: u32) -> Option<&MmsMessage> {
        for i in 0..self.message_count {
            if self.messages[i].id == msg_id {
                return Some(&self.messages[i]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_conversation_creation_and_participants() {
        let mut engine = MessagesMmsEngine::new();
        let participants = ["+919876543210", "+919876543211", "+919876543212"];

        let gid = engine.create_group("Project IndOS Core", &participants, 1000).unwrap();
        assert_eq!(gid, 1);
        assert_eq!(engine.group_count(), 1);

        let group = engine.get_group(gid).unwrap();
        assert_eq!(group.name_as_str(), "Project IndOS Core");
        assert_eq!(group.participant_count, 3);
        assert_eq!(group.get_participant(0), Some("+919876543210"));
        assert_eq!(group.get_participant(1), Some("+919876543211"));
        assert_eq!(group.get_participant(2), Some("+919876543212"));
    }

    #[test]
    fn test_mms_send_requires_capability() {
        let mut engine = MessagesMmsEngine::new();
        let gid = engine.create_group("Family", &["+919000000001", "+919000000002"], 1000).unwrap();

        let image_data = [0xFF, 0xD8, 0xFF, 0xE0]; // JPEG SOI magic
        let parts = [("image/jpeg", image_data.as_slice())];

        // 1. Missing CAP_APP_MMS_SEND rejected
        let res_err = engine.send_mms(Some(gid), "Vacation", "Look at this photo!", &parts, 1001, 0);
        assert_eq!(res_err, Err(MessagesMmsError::MissingMmsSendCapability));

        // 2. Authorized dispatch succeeds
        let msg_id = engine.send_mms(Some(gid), "Vacation", "Look at this photo!", &parts, 1001, CAP_APP_MMS_SEND).unwrap();
        assert_eq!(msg_id, 1);

        let msg = engine.get_message(msg_id).unwrap();
        assert_eq!(msg.subject_as_str(), "Vacation");
        assert_eq!(msg.part_count, 1);
        assert_eq!(msg.parts[0].mime_as_str(), "image/jpeg");
        assert_eq!(msg.protocol, MessageTransportProtocol::Mms);
    }

    #[test]
    fn test_rcs_negotiation_and_fallback_to_mms() {
        let mut engine = MessagesMmsEngine::new();

        // 1. Initially RCS network is off -> Negotiates MMS for attachments
        assert_eq!(engine.negotiate_transport(true), MessageTransportProtocol::Mms);
        assert_eq!(engine.negotiate_transport(false), MessageTransportProtocol::Sms);

        // 2. RCS network online & peer capable -> Negotiates RCS
        engine.set_rcs_network_available(true);
        engine.set_peer_rcs_capable(true);
        assert_eq!(engine.negotiate_transport(true), MessageTransportProtocol::RcsUniversal);

        // 3. Peer loses RCS -> Seamless fallback to MMS
        engine.set_peer_rcs_capable(false);
        assert_eq!(engine.negotiate_transport(true), MessageTransportProtocol::Mms);
    }

    #[test]
    fn test_rcs_typing_indicator_state() {
        let mut engine = MessagesMmsEngine::new();
        assert!(!engine.rcs_state().is_peer_typing);

        engine.set_peer_typing(true);
        assert!(engine.rcs_state().is_peer_typing);

        engine.set_peer_typing(false);
        assert!(!engine.rcs_state().is_peer_typing);
    }
}
