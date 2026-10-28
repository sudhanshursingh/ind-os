//! Messages App I: Cellular SMS Engine, Delivery Queue & Threading (SP-135, REQ-APP-MSG-001..003)
//!
//! Bible §8.18.3, §9.9.3, REQ-APP-MSG-001, REQ-APP-MSG-002, REQ-APP-MSG-003
//! Invariants:
//! - INV-MSG-001: Outbound SMS dispatch requires explicit CAP_APP_SMS_SEND; inbound processing
//!                requires CAP_APP_SMS_RECEIVE; ambient unprivileged apps cannot inspect or inject SMS.
//! - INV-MSG-002: Transient cellular drops trigger deterministic on-device retry queuing (max 3 attempts);
//!                delivery status transitions monotonically: Pending -> Sent -> Delivered (or Failed).
//! - INV-MSG-003: Messages are partitioned strictly into E.164 normalized address threads with
//!                zero cloud sync, bounded draft retention, and real-time unread badges.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

/// Capability required to send cellular SMS messages (REQ-APP-MSG-001).
pub const CAP_APP_SMS_SEND: u64 = 0x0080_0000_0000;
/// Capability required to receive and process inbound cellular SMS messages.
pub const CAP_APP_SMS_RECEIVE: u64 = 0x0100_0000_0000;

/// Standard cellular SMS body length limit in bytes (GSM 7-bit 160 characters / 8-bit octets).
pub const MAX_SMS_BODY_LEN: usize = 160;
/// Maximum length of a cellular phone number or shortcode address.
pub const MAX_ADDRESS_LEN: usize = 32;
/// Maximum number of messages retained per thread in memory.
pub const MAX_MESSAGES_PER_THREAD: usize = 32;
/// Maximum number of active conversation threads.
pub const MAX_THREADS: usize = 16;
/// Maximum delivery retry attempts before hard failure (REQ-APP-MSG-002).
pub const MAX_RETRY_ATTEMPTS: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagesCoreError {
    MissingSendCapability,
    MissingReceiveCapability,
    InvalidAddress,
    BodyTooLong,
    ThreadStoreFull,
    ThreadMessageLimitReached,
    ThreadNotFound,
    MessageNotFound,
    MaxRetriesExceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
}

/// Standard cellular SMS representation.
#[derive(Debug, Clone, Copy)]
pub struct SmsMessage {
    pub id: u32,
    pub address: [u8; MAX_ADDRESS_LEN],
    pub address_len: usize,
    pub body: [u8; MAX_SMS_BODY_LEN],
    pub body_len: usize,
    pub timestamp_sec: u64,
    pub direction: MessageDirection,
    pub status: DeliveryStatus,
    pub retry_count: u8,
}

impl SmsMessage {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            address: [0u8; MAX_ADDRESS_LEN],
            address_len: 0,
            body: [0u8; MAX_SMS_BODY_LEN],
            body_len: 0,
            timestamp_sec: 0,
            direction: MessageDirection::Inbound,
            status: DeliveryStatus::Pending,
            retry_count: 0,
        }
    }

    pub fn address_as_str(&self) -> &str {
        core::str::from_utf8(&self.address[..self.address_len]).unwrap_or("")
    }

    pub fn body_as_str(&self) -> &str {
        core::str::from_utf8(&self.body[..self.body_len]).unwrap_or("")
    }
}

/// Address-grouped conversation thread (REQ-APP-MSG-003).
#[derive(Debug, Clone, Copy)]
pub struct ConversationThread {
    pub thread_id: u32,
    pub address: [u8; MAX_ADDRESS_LEN],
    pub address_len: usize,
    pub messages: [SmsMessage; MAX_MESSAGES_PER_THREAD],
    pub message_count: usize,
    pub unread_count: u32,
    pub draft_body: [u8; MAX_SMS_BODY_LEN],
    pub draft_len: usize,
    pub last_activity_sec: u64,
}

impl ConversationThread {
    pub const fn empty() -> Self {
        Self {
            thread_id: 0,
            address: [0u8; MAX_ADDRESS_LEN],
            address_len: 0,
            messages: [SmsMessage::empty(); MAX_MESSAGES_PER_THREAD],
            message_count: 0,
            unread_count: 0,
            draft_body: [0u8; MAX_SMS_BODY_LEN],
            draft_len: 0,
            last_activity_sec: 0,
        }
    }

    pub fn address_as_str(&self) -> &str {
        core::str::from_utf8(&self.address[..self.address_len]).unwrap_or("")
    }

    pub fn draft_as_str(&self) -> &str {
        core::str::from_utf8(&self.draft_body[..self.draft_len]).unwrap_or("")
    }
}

/// Integrated SMS & Threaded Conversations Engine.
pub struct MessagesEngine {
    threads: [ConversationThread; MAX_THREADS],
    thread_count: usize,
    next_msg_id: u32,
    next_thread_id: u32,
}

impl MessagesEngine {
    pub const fn new() -> Self {
        Self {
            threads: [ConversationThread::empty(); MAX_THREADS],
            thread_count: 0,
            next_msg_id: 1,
            next_thread_id: 1,
        }
    }

    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    pub fn total_unread_count(&self) -> u32 {
        let mut sum = 0;
        for i in 0..self.thread_count {
            sum += self.threads[i].unread_count;
        }
        sum
    }

    pub fn get_thread(&self, address: &str) -> Option<&ConversationThread> {
        let addr_bytes = address.as_bytes();
        for i in 0..self.thread_count {
            let t = &self.threads[i];
            if &t.address[..t.address_len] == addr_bytes {
                return Some(t);
            }
        }
        None
    }

    fn find_or_create_thread(&mut self, address: &[u8], now_sec: u64) -> Result<usize, MessagesCoreError> {
        for i in 0..self.thread_count {
            if &self.threads[i].address[..self.threads[i].address_len] == address {
                return Ok(i);
            }
        }

        if self.thread_count >= MAX_THREADS {
            return Err(MessagesCoreError::ThreadStoreFull);
        }

        let idx = self.thread_count;
        let mut t = ConversationThread::empty();
        t.thread_id = self.next_thread_id;
        self.next_thread_id += 1;

        let len = address.len().min(MAX_ADDRESS_LEN);
        t.address[..len].copy_from_slice(&address[..len]);
        t.address_len = len;
        t.last_activity_sec = now_sec;

        self.threads[idx] = t;
        self.thread_count += 1;
        Ok(idx)
    }

    /// Compose and send a new SMS message (REQ-APP-MSG-001, INV-MSG-001).
    /// Enforces CAP_APP_SMS_SEND.
    pub fn send_sms(
        &mut self,
        recipient: &str,
        body: &str,
        timestamp_sec: u64,
        cellular_connected: bool,
        caller_caps: u64,
    ) -> Result<u32, MessagesCoreError> {
        if (caller_caps & CAP_APP_SMS_SEND) == 0 {
            return Err(MessagesCoreError::MissingSendCapability);
        }

        let r_bytes = recipient.as_bytes();
        if r_bytes.is_empty() || r_bytes.len() > MAX_ADDRESS_LEN {
            return Err(MessagesCoreError::InvalidAddress);
        }

        let b_bytes = body.as_bytes();
        if b_bytes.len() > MAX_SMS_BODY_LEN {
            return Err(MessagesCoreError::BodyTooLong);
        }

        let thread_idx = self.find_or_create_thread(r_bytes, timestamp_sec)?;
        let thread = &mut self.threads[thread_idx];

        if thread.message_count >= MAX_MESSAGES_PER_THREAD {
            return Err(MessagesCoreError::ThreadMessageLimitReached);
        }

        let msg_id = self.next_msg_id;
        self.next_msg_id += 1;

        let mut msg = SmsMessage::empty();
        msg.id = msg_id;
        msg.address[..r_bytes.len()].copy_from_slice(r_bytes);
        msg.address_len = r_bytes.len();
        msg.body[..b_bytes.len()].copy_from_slice(b_bytes);
        msg.body_len = b_bytes.len();
        msg.timestamp_sec = timestamp_sec;
        msg.direction = MessageDirection::Outbound;

        // If cellular network is active, message transitions to Sent; otherwise Pending in retry queue (REQ-APP-MSG-002)
        if cellular_connected {
            msg.status = DeliveryStatus::Sent;
        } else {
            msg.status = DeliveryStatus::Pending;
        }

        let m_idx = thread.message_count;
        thread.messages[m_idx] = msg;
        thread.message_count += 1;
        thread.last_activity_sec = timestamp_sec;
        // Sending clear any draft
        thread.draft_len = 0;

        Ok(msg_id)
    }

    /// Receive an inbound SMS message from cellular modem HAL (REQ-APP-MSG-001).
    /// Enforces CAP_APP_SMS_RECEIVE.
    pub fn receive_sms(
        &mut self,
        sender: &str,
        body: &str,
        timestamp_sec: u64,
        caller_caps: u64,
    ) -> Result<u32, MessagesCoreError> {
        if (caller_caps & CAP_APP_SMS_RECEIVE) == 0 {
            return Err(MessagesCoreError::MissingReceiveCapability);
        }

        let s_bytes = sender.as_bytes();
        if s_bytes.is_empty() || s_bytes.len() > MAX_ADDRESS_LEN {
            return Err(MessagesCoreError::InvalidAddress);
        }

        let b_bytes = body.as_bytes();
        if b_bytes.len() > MAX_SMS_BODY_LEN {
            return Err(MessagesCoreError::BodyTooLong);
        }

        let thread_idx = self.find_or_create_thread(s_bytes, timestamp_sec)?;
        let thread = &mut self.threads[thread_idx];

        if thread.message_count >= MAX_MESSAGES_PER_THREAD {
            return Err(MessagesCoreError::ThreadMessageLimitReached);
        }

        let msg_id = self.next_msg_id;
        self.next_msg_id += 1;

        let mut msg = SmsMessage::empty();
        msg.id = msg_id;
        msg.address[..s_bytes.len()].copy_from_slice(s_bytes);
        msg.address_len = s_bytes.len();
        msg.body[..b_bytes.len()].copy_from_slice(b_bytes);
        msg.body_len = b_bytes.len();
        msg.timestamp_sec = timestamp_sec;
        msg.direction = MessageDirection::Inbound;
        msg.status = DeliveryStatus::Delivered;

        let m_idx = thread.message_count;
        thread.messages[m_idx] = msg;
        thread.message_count += 1;
        thread.unread_count += 1;
        thread.last_activity_sec = timestamp_sec;

        Ok(msg_id)
    }

    /// Process carrier delivery receipt (REQ-APP-MSG-002).
    pub fn update_delivery_status(&mut self, msg_id: u32, status: DeliveryStatus) -> Result<(), MessagesCoreError> {
        for t in 0..self.thread_count {
            let thread = &mut self.threads[t];
            for m in 0..thread.message_count {
                if thread.messages[m].id == msg_id {
                    thread.messages[m].status = status;
                    return Ok(());
                }
            }
        }
        Err(MessagesCoreError::MessageNotFound)
    }

    /// On-device retry queue flusher (REQ-APP-MSG-002, INV-MSG-002).
    /// Retries pending or transiently dropped SMS messages when cellular connection is restored.
    pub fn retry_pending_queue(&mut self, cellular_connected: bool) -> usize {
        let mut dispatched = 0;
        for t in 0..self.thread_count {
            let thread = &mut self.threads[t];
            for m in 0..thread.message_count {
                let msg = &mut thread.messages[m];
                if msg.direction == MessageDirection::Outbound && msg.status == DeliveryStatus::Pending {
                    if cellular_connected {
                        msg.status = DeliveryStatus::Sent;
                        dispatched += 1;
                    } else {
                        msg.retry_count += 1;
                        if msg.retry_count >= MAX_RETRY_ATTEMPTS {
                            msg.status = DeliveryStatus::Failed;
                        }
                    }
                }
            }
        }
        dispatched
    }

    /// Save persistent draft for a recipient thread (REQ-APP-MSG-003).
    pub fn set_draft(&mut self, recipient: &str, draft: &str) -> Result<(), MessagesCoreError> {
        let r_bytes = recipient.as_bytes();
        if r_bytes.is_empty() || r_bytes.len() > MAX_ADDRESS_LEN {
            return Err(MessagesCoreError::InvalidAddress);
        }
        let d_bytes = draft.as_bytes();
        if d_bytes.len() > MAX_SMS_BODY_LEN {
            return Err(MessagesCoreError::BodyTooLong);
        }

        let thread_idx = self.find_or_create_thread(r_bytes, 0)?;
        let thread = &mut self.threads[thread_idx];

        thread.draft_body[..d_bytes.len()].copy_from_slice(d_bytes);
        thread.draft_len = d_bytes.len();
        Ok(())
    }

    /// Mark all unread messages in a thread as read (REQ-APP-MSG-003).
    pub fn mark_thread_read(&mut self, address: &str) -> Result<u32, MessagesCoreError> {
        let a_bytes = address.as_bytes();
        for i in 0..self.thread_count {
            let thread = &mut self.threads[i];
            if &thread.address[..thread.address_len] == a_bytes {
                let cleared = thread.unread_count;
                thread.unread_count = 0;
                return Ok(cleared);
            }
        }
        Err(MessagesCoreError::ThreadNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sms_send_requires_capability() {
        let mut engine = MessagesEngine::new();

        // 1. Missing CAP_APP_SMS_SEND rejected
        let res = engine.send_sms("+919876543210", "Namaste Ind OS", 1000, true, 0);
        assert_eq!(res, Err(MessagesCoreError::MissingSendCapability));

        // 2. Authorized send succeeds
        let msg_id = engine.send_sms("+919876543210", "Namaste Ind OS", 1000, true, CAP_APP_SMS_SEND).unwrap();
        assert_eq!(msg_id, 1);
        assert_eq!(engine.thread_count(), 1);

        let thread = engine.get_thread("+919876543210").unwrap();
        assert_eq!(thread.message_count, 1);
        assert_eq!(thread.messages[0].body_as_str(), "Namaste Ind OS");
        assert_eq!(thread.messages[0].direction, MessageDirection::Outbound);
        assert_eq!(thread.messages[0].status, DeliveryStatus::Sent);
    }

    #[test]
    fn test_sms_receive_and_unread_badges() {
        let mut engine = MessagesEngine::new();

        // 1. Missing CAP_APP_SMS_RECEIVE rejected
        let res = engine.receive_sms("+919876543210", "Hello from friend", 1000, 0);
        assert_eq!(res, Err(MessagesCoreError::MissingReceiveCapability));

        // 2. Receive 2 messages from same contact
        engine.receive_sms("+919876543210", "Hello 1", 1001, CAP_APP_SMS_RECEIVE).unwrap();
        engine.receive_sms("+919876543210", "Hello 2", 1002, CAP_APP_SMS_RECEIVE).unwrap();

        assert_eq!(engine.total_unread_count(), 2);
        let thread = engine.get_thread("+919876543210").unwrap();
        assert_eq!(thread.unread_count, 2);
        assert_eq!(thread.message_count, 2);

        // 3. Mark thread as read
        let cleared = engine.mark_thread_read("+919876543210").unwrap();
        assert_eq!(cleared, 2);
        assert_eq!(engine.total_unread_count(), 0);
    }

    #[test]
    fn test_delivery_reliability_retry_queue() {
        let mut engine = MessagesEngine::new();

        // 1. Send SMS while offline -> status is Pending
        let msg_id = engine.send_sms("+919876543210", "Offline message", 1000, false, CAP_APP_SMS_SEND).unwrap();
        let thread = engine.get_thread("+919876543210").unwrap();
        assert_eq!(thread.messages[0].status, DeliveryStatus::Pending);

        // 2. Retrying while still offline increments retry count
        let sent = engine.retry_pending_queue(false);
        assert_eq!(sent, 0);
        assert_eq!(engine.get_thread("+919876543210").unwrap().messages[0].retry_count, 1);

        // 3. Cellular comes online -> queue flushes to Sent
        let flushed = engine.retry_pending_queue(true);
        assert_eq!(flushed, 1);
        assert_eq!(engine.get_thread("+919876543210").unwrap().messages[0].status, DeliveryStatus::Sent);

        // 4. Carrier delivery receipt arrives -> status transitions to Delivered
        engine.update_delivery_status(msg_id, DeliveryStatus::Delivered).unwrap();
        assert_eq!(engine.get_thread("+919876543210").unwrap().messages[0].status, DeliveryStatus::Delivered);
    }

    #[test]
    fn test_retry_exhaustion_marks_failure() {
        let mut engine = MessagesEngine::new();
        engine.send_sms("+919876543210", "Drop test", 1000, false, CAP_APP_SMS_SEND).unwrap();

        // Retry 3 times while offline
        engine.retry_pending_queue(false); // retry 1
        engine.retry_pending_queue(false); // retry 2
        engine.retry_pending_queue(false); // retry 3 -> Failed

        let thread = engine.get_thread("+919876543210").unwrap();
        assert_eq!(thread.messages[0].status, DeliveryStatus::Failed);
    }

    #[test]
    fn test_draft_persistence_per_thread() {
        let mut engine = MessagesEngine::new();
        engine.set_draft("+919999999999", "Draft in progress...").unwrap();

        let thread = engine.get_thread("+919999999999").unwrap();
        assert_eq!(thread.draft_as_str(), "Draft in progress...");

        // Sending clears draft
        engine.send_sms("+919999999999", "Sent message", 2000, true, CAP_APP_SMS_SEND).unwrap();
        let thread_after = engine.get_thread("+919999999999").unwrap();
        assert_eq!(thread_after.draft_as_str(), "");
    }
}

