//! Messages App II: Suraksha Security, Screen-Share Blackout & Attachments (SP-136, REQ-APP-MSG-004..006, 011)
//!
//! Bible §8.18.3, §9.9.3, REQ-APP-MSG-004, REQ-APP-MSG-005, REQ-APP-MSG-006, REQ-APP-MSG-011
//! Invariants:
//! - INV-MSG-004: Attachments are mediated strictly via OS share sheet/picker capability token
//!                (CAP_ATTACHMENT_MEDIATE); ambient filesystem scanning is strictly prohibited.
//! - INV-MSG-005: Banking OTP messages are detected and redacted into blackout mask during active
//!                screen-share/mirroring; forwarded only with explicit one-shot user consent.
//! - INV-MSG-006: Local on-device Suraksha classifier analyzes SMS against offline threat patterns
//!                (Digital Arrest, KYC Expiry, Electricity Disconnection, Lottery Fraud) with zero cloud queries.
//! - INV-MSG-007: Message store encryption at rest utilizes device-derived key envelopes (INV-MSG-011).
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

/// Capability required to attach files mediated via OS share sheet (REQ-APP-MSG-004).
pub const CAP_ATTACHMENT_MEDIATE: u64 = 0x0200_0000_0000;

/// Standard redaction string for OTPs during active screen sharing (REQ-APP-MSG-005).
pub const SCREEN_SHARE_BLACKOUT_STR: &str = "•••••• [PROTECTED BY SURAKSHA]";

/// Maximum attachment MIME type length.
pub const MAX_MIME_LEN: usize = 32;
/// Maximum payload size in encrypted envelope.
pub const MAX_ENVELOPE_LEN: usize = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagesSurakshaError {
    MissingMediationCapability,
    InvalidMimeType,
    AttachmentTooLarge,
    DecryptionFailed,
    InvalidDeviceKey,
}

/// Attachment descriptor granted strictly via OS file picker or share sheet (REQ-APP-MSG-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttachmentDescriptor {
    pub attachment_id: u32,
    pub mime_type: [u8; MAX_MIME_LEN],
    pub mime_len: usize,
    pub file_token: u64,
    pub size_bytes: usize,
}

impl AttachmentDescriptor {
    pub fn mime_as_str(&self) -> &str {
        core::str::from_utf8(&self.mime_type[..self.mime_len]).unwrap_or("")
    }
}

/// Banking / Authentication One-Time Password descriptor (REQ-APP-MSG-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OtpDescriptor {
    pub is_otp: bool,
    pub issuer: [u8; 16],
    pub issuer_len: usize,
    pub otp_code: [u8; 8],
    pub otp_len: usize,
}

impl OtpDescriptor {
    pub fn issuer_as_str(&self) -> &str {
        core::str::from_utf8(&self.issuer[..self.issuer_len]).unwrap_or("")
    }

    pub fn otp_as_str(&self) -> &str {
        core::str::from_utf8(&self.otp_code[..self.otp_len]).unwrap_or("")
    }
}

/// Suraksha offline SMS scam threat classification (REQ-APP-MSG-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmsScamVerdict {
    Safe,
    Suspicious { pattern_id: u16, alert: &'static str },
    CriticalScam { pattern_id: u16, alert: &'static str },
}

/// Encrypted message envelope at rest (REQ-APP-MSG-011).
#[derive(Debug, Clone, Copy)]
pub struct DeviceCipherEnvelope {
    pub ciphertext: [u8; MAX_ENVELOPE_LEN],
    pub cipher_len: usize,
    pub auth_tag: u32,
    pub key_epoch: u32,
}

/// Integrated Suraksha Security, Screen-Share Blackout & Attachments Engine.
pub struct MessagesSurakshaEngine {
    next_attachment_id: u32,
    screen_share_active: bool,
    device_key_epoch: u32,
}

impl MessagesSurakshaEngine {
    pub const fn new() -> Self {
        Self {
            next_attachment_id: 1,
            screen_share_active: false,
            device_key_epoch: 1,
        }
    }

    pub fn is_screen_share_active(&self) -> bool {
        self.screen_share_active
    }

    pub fn set_screen_share_active(&mut self, active: bool) {
        self.screen_share_active = active;
    }

    /// Attach a user-mediated file via capability token (REQ-APP-MSG-004, INV-MSG-004).
    /// Ambient file paths without CAP_ATTACHMENT_MEDIATE are strictly rejected.
    pub fn attach_file(
        &mut self,
        file_token: u64,
        mime_str: &str,
        size_bytes: usize,
        caller_caps: u64,
    ) -> Result<AttachmentDescriptor, MessagesSurakshaError> {
        if (caller_caps & CAP_ATTACHMENT_MEDIATE) == 0 {
            return Err(MessagesSurakshaError::MissingMediationCapability);
        }
        if file_token == 0 {
            return Err(MessagesSurakshaError::MissingMediationCapability);
        }
        if mime_str.is_empty() || mime_str.len() > MAX_MIME_LEN {
            return Err(MessagesSurakshaError::InvalidMimeType);
        }
        // Limit MMS/attachment to 10 MB under memory envelope
        if size_bytes > 10 * 1024 * 1024 {
            return Err(MessagesSurakshaError::AttachmentTooLarge);
        }

        let att_id = self.next_attachment_id;
        self.next_attachment_id += 1;

        let mut mime_buf = [0u8; MAX_MIME_LEN];
        let bytes = mime_str.as_bytes();
        let len = bytes.len().min(MAX_MIME_LEN);
        mime_buf[..len].copy_from_slice(&bytes[..len]);

        Ok(AttachmentDescriptor {
            attachment_id: att_id,
            mime_type: mime_buf,
            mime_len: len,
            file_token,
            size_bytes,
        })
    }

    /// On-device classification of incoming SMS for scam heuristics and OTP detection (REQ-APP-MSG-005, REQ-APP-MSG-006).
    pub fn inspect_incoming_sms(&self, sender: &str, body: &str) -> (SmsScamVerdict, Option<OtpDescriptor>) {
        let mut otp = None;
        let mut verdict = SmsScamVerdict::Safe;

        let body_bytes = body.as_bytes();

        // 1. Detect Banking OTP
        // Common Indian bank headers (e.g. VM-HDFCBK, AX-SBIINB, BZ-ICICIB, AD-AXISBK) or standard OTP keywords
        let is_otp_keyword = Self::contains_subseq(body_bytes, b"OTP")
            || Self::contains_subseq(body_bytes, b"otp")
            || Self::contains_subseq(body_bytes, b"one time password")
            || Self::contains_subseq(body_bytes, b"verification code");

        if is_otp_keyword {
            // Extract numeric digits of OTP (typically 4-8 digits)
            let mut code_buf = [0u8; 8];
            let mut code_len = 0;
            let mut current_digits = 0;
            let mut candidate_buf = [0u8; 8];

            for &b in body_bytes {
                if b.is_ascii_digit() {
                    if current_digits < 8 {
                        candidate_buf[current_digits] = b;
                        current_digits += 1;
                    }
                } else {
                    if current_digits >= 4 && current_digits <= 8 {
                        code_buf[..current_digits].copy_from_slice(&candidate_buf[..current_digits]);
                        code_len = current_digits;
                        break;
                    }
                    current_digits = 0;
                }
            }

            if code_len == 0 && current_digits >= 4 && current_digits <= 8 {
                code_buf[..current_digits].copy_from_slice(&candidate_buf[..current_digits]);
                code_len = current_digits;
            }

            let mut issuer_buf = [0u8; 16];
            let s_bytes = sender.as_bytes();
            let s_len = s_bytes.len().min(16);
            issuer_buf[..s_len].copy_from_slice(&s_bytes[..s_len]);

            otp = Some(OtpDescriptor {
                is_otp: true,
                issuer: issuer_buf,
                issuer_len: s_len,
                otp_code: code_buf,
                otp_len: code_len,
            });
        }

        // 2. Offline Suraksha Scam Heuristics (REQ-APP-MSG-006)
        // Pattern 1: Digital Arrest / Police / CBI Extortion
        if Self::contains_subseq(body_bytes, b"digital arrest")
            || Self::contains_subseq(body_bytes, b"cbi narcotics")
            || Self::contains_subseq(body_bytes, b"police arrest warrant")
        {
            verdict = SmsScamVerdict::CriticalScam {
                pattern_id: 101,
                alert: "CRITICAL: Fake 'Digital Arrest' extortion detected. Police never arrest via phone.",
            };
        }
        // Pattern 2: Electricity disconnection scam
        else if Self::contains_subseq(body_bytes, b"electricity will be disconnected")
            || Self::contains_subseq(body_bytes, b"power will be disconnected tonight")
        {
            verdict = SmsScamVerdict::CriticalScam {
                pattern_id: 102,
                alert: "SCAM ALERT: Electricity bill phishing fraud. Do not call numbers in message.",
            };
        }
        // Pattern 3: Bank KYC suspension scam
        else if Self::contains_subseq(body_bytes, b"kyc expired")
            || Self::contains_subseq(body_bytes, b"account blocked update pan")
            || Self::contains_subseq(body_bytes, b"sim blocked today")
        {
            verdict = SmsScamVerdict::CriticalScam {
                pattern_id: 103,
                alert: "PHISHING ALERT: Fake KYC/account suspension alert. Banks never ask PAN via SMS links.",
            };
        }
        // Pattern 4: Lottery / Prize / Job offer scams
        else if Self::contains_subseq(body_bytes, b"won lottery")
            || Self::contains_subseq(body_bytes, b"part time job 5000 daily")
        {
            verdict = SmsScamVerdict::Suspicious {
                pattern_id: 201,
                alert: "WARNING: High-probability unsolicited prize/task scam.",
            };
        }

        (verdict, otp)
    }

    /// Renders SMS text for visual presentation.
    /// If screen sharing is active and the message contains an OTP, the OTP code is blacked out (REQ-APP-MSG-005, INV-MSG-005).
    pub fn render_for_display<'a>(
        &self,
        body: &'a str,
        otp_opt: Option<&OtpDescriptor>,
    ) -> &'a str {
        if self.screen_share_active {
            if let Some(otp) = otp_opt {
                if otp.is_otp {
                    return SCREEN_SHARE_BLACKOUT_STR;
                }
            }
        }
        body
    }

    /// Device-derived encryption envelope at rest (REQ-APP-MSG-011, INV-MSG-007).
    pub fn encrypt_at_rest(&self, plaintext: &[u8], device_key: u64) -> Result<DeviceCipherEnvelope, MessagesSurakshaError> {
        if device_key == 0 {
            return Err(MessagesSurakshaError::InvalidDeviceKey);
        }
        if plaintext.len() > MAX_ENVELOPE_LEN {
            return Err(MessagesSurakshaError::AttachmentTooLarge);
        }

        let mut cipher = [0u8; MAX_ENVELOPE_LEN];
        let mut checksum: u32 = (device_key as u32) ^ ((device_key >> 32) as u32) ^ 0x811c9dc5;

        // Stream cipher keystream generator using device-derived key and epoch
        for (i, &b) in plaintext.iter().enumerate() {
            let key_byte = (((device_key >> ((i % 8) * 8)) & 0xFF) as u8) ^ ((i as u8).wrapping_mul(31));
            let enc = b ^ key_byte;
            cipher[i] = enc;
            checksum ^= enc as u32;
            checksum = checksum.wrapping_mul(0x01000193);
        }

        Ok(DeviceCipherEnvelope {
            ciphertext: cipher,
            cipher_len: plaintext.len(),
            auth_tag: checksum,
            key_epoch: self.device_key_epoch,
        })
    }

    /// Decrypt message envelope from storage (REQ-APP-MSG-011).
    pub fn decrypt_at_rest(&self, envelope: &DeviceCipherEnvelope, device_key: u64) -> Result<[u8; MAX_ENVELOPE_LEN], MessagesSurakshaError> {
        if device_key == 0 {
            return Err(MessagesSurakshaError::InvalidDeviceKey);
        }

        let mut checksum: u32 = (device_key as u32) ^ ((device_key >> 32) as u32) ^ 0x811c9dc5;
        let mut plain = [0u8; MAX_ENVELOPE_LEN];

        for i in 0..envelope.cipher_len {
            let enc = envelope.ciphertext[i];
            checksum ^= enc as u32;
            checksum = checksum.wrapping_mul(0x01000193);

            let key_byte = (((device_key >> ((i % 8) * 8)) & 0xFF) as u8) ^ ((i as u8).wrapping_mul(31));
            plain[i] = enc ^ key_byte;
        }

        if checksum != envelope.auth_tag {
            return Err(MessagesSurakshaError::DecryptionFailed);
        }

        Ok(plain)
    }

    fn contains_subseq(haystack: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() || haystack.len() < needle.len() {
            return false;
        }
        for i in 0..=haystack.len() - needle.len() {
            let mut matches = true;
            for j in 0..needle.len() {
                if haystack[i + j].to_ascii_lowercase() != needle[j].to_ascii_lowercase() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_mediation_capability_check() {
        let mut engine = MessagesSurakshaEngine::new();

        // 1. Missing CAP_ATTACHMENT_MEDIATE rejected
        let res_err = engine.attach_file(1001, "image/png", 50_000, 0);
        assert_eq!(res_err, Err(MessagesSurakshaError::MissingMediationCapability));

        // 2. Zero file token rejected
        let zero_token = engine.attach_file(0, "image/png", 50_000, CAP_ATTACHMENT_MEDIATE);
        assert_eq!(zero_token, Err(MessagesSurakshaError::MissingMediationCapability));

        // 3. Authorized attachment succeeds
        let att = engine.attach_file(1001, "image/png", 50_000, CAP_ATTACHMENT_MEDIATE).unwrap();
        assert_eq!(att.attachment_id, 1);
        assert_eq!(att.mime_as_str(), "image/png");
        assert_eq!(att.file_token, 1001);
    }

    #[test]
    fn test_banking_otp_detection_and_screen_share_blackout() {
        let mut engine = MessagesSurakshaEngine::new();

        let sender = "VK-HDFCBK";
        let body = "Your HDFC Bank OTP is 749201 for INR 5000.00 to merchant ABC. Valid for 10 mins. Do not share.";

        let (_verdict, otp_opt) = engine.inspect_incoming_sms(sender, body);
        assert!(otp_opt.is_some());
        let otp = otp_opt.unwrap();
        assert!(otp.is_otp);
        assert_eq!(otp.otp_as_str(), "749201");
        assert_eq!(otp.issuer_as_str(), "VK-HDFCBK");

        // Normal display when screen share is off -> visible
        let visible = engine.render_for_display(body, Some(&otp));
        assert_eq!(visible, body);

        // Screen share is active -> blacked out with Suraksha mask
        engine.set_screen_share_active(true);
        let blacked_out = engine.render_for_display(body, Some(&otp));
        assert_eq!(blacked_out, SCREEN_SHARE_BLACKOUT_STR);

        // Regular non-OTP message is never blacked out
        let normal_msg = "Hello, meeting at 5pm.";
        let (verdict_normal, otp_none) = engine.inspect_incoming_sms("+919876543210", normal_msg);
        assert_eq!(verdict_normal, SmsScamVerdict::Safe);
        assert!(otp_none.is_none());
        assert_eq!(engine.render_for_display(normal_msg, None), normal_msg);
    }

    #[test]
    fn test_suraksha_offline_scam_heuristics() {
        let engine = MessagesSurakshaEngine::new();

        // 1. Digital arrest scam
        let (v1, _) = engine.inspect_incoming_sms("+919999999999", "URGENT: You are under digital arrest by CBI Narcotics branch.");
        assert_eq!(
            v1,
            SmsScamVerdict::CriticalScam {
                pattern_id: 101,
                alert: "CRITICAL: Fake 'Digital Arrest' extortion detected. Police never arrest via phone.",
            }
        );

        // 2. Electricity bill scam
        let (v2, _) = engine.inspect_incoming_sms("+918888888888", "Dear Consumer, your power will be disconnected tonight. Call officer immediately.");
        assert_eq!(
            v2,
            SmsScamVerdict::CriticalScam {
                pattern_id: 102,
                alert: "SCAM ALERT: Electricity bill phishing fraud. Do not call numbers in message.",
            }
        );

        // 3. Bank KYC expiry scam
        let (v3, _) = engine.inspect_incoming_sms("+917777777777", "Your SBI bank account blocked update pan card now or sim blocked today.");
        assert_eq!(
            v3,
            SmsScamVerdict::CriticalScam {
                pattern_id: 103,
                alert: "PHISHING ALERT: Fake KYC/account suspension alert. Banks never ask PAN via SMS links.",
            }
        );

        // 4. Safe message
        let (v4, _) = engine.inspect_incoming_sms("+919876543210", "See you for dinner tonight!");
        assert_eq!(v4, SmsScamVerdict::Safe);
    }

    #[test]
    fn test_message_encryption_at_rest() {
        let engine = MessagesSurakshaEngine::new();
        let device_key: u64 = 0xA5A5_5A5A_1234_5678;
        let plaintext = b"Confidential personal conversation.";

        // Encrypt message
        let envelope = engine.encrypt_at_rest(plaintext, device_key).unwrap();
        assert_ne!(&envelope.ciphertext[..envelope.cipher_len], plaintext);

        // Decrypt with correct device key
        let decrypted = engine.decrypt_at_rest(&envelope, device_key).unwrap();
        assert_eq!(&decrypted[..envelope.cipher_len], plaintext);

        // Decrypt with wrong key fails integrity check
        let wrong_key: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let err = engine.decrypt_at_rest(&envelope, wrong_key);
        assert_eq!(err, Err(MessagesSurakshaError::DecryptionFailed));
    }
}
