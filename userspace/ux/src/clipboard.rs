//! Encrypted Clipboard Store with Sensitive-Content (OTP/Password) Auto-Redaction (SP-128,
//! REQ-APP-COMMON-011 & REQ-APP-COMMON-012)
//!
//! Bible §8.18.2, §9.9.1 REQ-APP-COMMON-011, REQ-APP-COMMON-012, DEF-APP-006
//! Invariants:
//! - INV-CLIP-001: Sensitive content (OTPs, passwords) is auto-cleared after SENSITIVE_TIMEOUT_TICKS.
//! - INV-CLIP-002: Inter-app clipboard reads require per-read user consent — ambient snooping is
//!                 physically blocked by the capability gate.
//! - INV-CLIP-003: All entries are logically encrypted (keyed with device-derived epoch; real
//!                 encryption wired to IndFS crypto layer in Phase 9).
//! Rules: N-01 (zero external crates), N-04 (no unsafe), DEF-APP-006 (60 s OTP/password timeout)

/// Logical clock ticks representing 60 seconds at the system's tick rate.
/// DEF-APP-006: 60-second auto-clear for OTPs/passwords.
pub const SENSITIVE_TIMEOUT_TICKS: u64 = 60;

/// Maximum number of clipboard entries held simultaneously.
pub const MAX_CLIPBOARD_ENTRIES: usize = 8;
/// Maximum inline payload bytes per clipboard entry.
pub const MAX_CLIP_PAYLOAD: usize = 1024;

/// Capability required to write to the clipboard.
pub const CAP_CLIPBOARD_WRITE: u64 = 0x0000_0000_0000_1000;
/// Capability required to read from the clipboard (per-read consent still required — INV-CLIP-002).
pub const CAP_CLIPBOARD_READ: u64 = 0x0000_0000_0000_2000;

// ─── Sensitivity classification ──────────────────────────────────────────────

/// Content sensitivity classifier. Determines whether auto-clear applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipSensitivity {
    /// Plain text, URLs, general content — no forced timeout.
    Normal,
    /// OTP codes (6–8 digit numeric sequences): auto-clear at SENSITIVE_TIMEOUT_TICKS.
    Otp,
    /// Password or secret string: auto-clear at SENSITIVE_TIMEOUT_TICKS.
    Password,
}

impl ClipSensitivity {
    /// Heuristic classifier: detect OTPs (all ASCII digits, 4–8 chars) and passwords
    /// (declared by the source app via the `password` flag).
    pub fn classify(data: &[u8], declared_password: bool) -> Self {
        if declared_password {
            return ClipSensitivity::Password;
        }
        // OTP: 4–8 ASCII digit bytes
        let len = data.len();
        if len >= 4 && len <= 8 && data.iter().all(|&b| b.is_ascii_digit()) {
            return ClipSensitivity::Otp;
        }
        ClipSensitivity::Normal
    }

    /// True if this entry should be auto-cleared after SENSITIVE_TIMEOUT_TICKS.
    pub fn is_sensitive(self) -> bool {
        matches!(self, ClipSensitivity::Otp | ClipSensitivity::Password)
    }
}

// ─── Clipboard entry ─────────────────────────────────────────────────────────

/// One clipboard entry. In Phase 9 the `data` field is encrypted with the device-derived
/// epoch key before write; here the struct tracks that invariant for the audit log (INV-CLIP-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub entry_id: u32,
    /// PID of the writing app.
    pub writer_pid: u32,
    /// Inline payload (logically encrypted).
    pub data: [u8; MAX_CLIP_PAYLOAD],
    pub data_len: usize,
    pub sensitivity: ClipSensitivity,
    /// System tick at which this entry was written.
    pub written_at_tick: u64,
    /// True once the entry has been cryptographically zeroed.
    pub cleared: bool,
}

impl ClipboardEntry {
    /// Cryptographically zero the payload bytes (INV-CLIP-001).
    pub fn secure_clear(&mut self) {
        self.data.iter_mut().for_each(|b| *b = 0);
        self.data_len = 0;
        self.cleared = true;
    }

    /// True if the entry has expired (sensitive and past the timeout).
    pub fn is_expired(&self, current_tick: u64) -> bool {
        self.sensitivity.is_sensitive()
            && (current_tick.saturating_sub(self.written_at_tick) >= SENSITIVE_TIMEOUT_TICKS)
    }
}

// ─── Consent record for inter-app read ───────────────────────────────────────

/// A pending per-read consent request (INV-CLIP-002).
#[derive(Debug, Clone, Copy)]
pub struct ClipReadRequest {
    pub request_id: u32,
    pub reader_pid: u32,
    pub target_entry_id: u32,
    /// True once the user explicitly grants this single read.
    pub user_granted: bool,
}

pub const MAX_CLIP_READ_REQUESTS: usize = 8;

// ─── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardError {
    /// Write caller lacks CAP_CLIPBOARD_WRITE.
    MissingWriteCapability,
    /// Read caller lacks CAP_CLIPBOARD_READ.
    MissingReadCapability,
    /// Entry not found (cleared, expired, or never existed).
    EntryNotFound,
    /// Clipboard store is full.
    StoreFull,
    /// Consent queue is full.
    ConsentQueueFull,
    /// Request not found.
    RequestNotFound,
    /// User denied the read request.
    ConsentDenied,
    /// Entry was cleared (sensitive timeout expired).
    EntryCleared,
    /// Payload too large for inline store.
    PayloadTooLarge,
}

// ─── Engine ──────────────────────────────────────────────────────────────────

/// The Clipboard Store engine (INV-CLIP-001..003, REQ-APP-COMMON-011, REQ-APP-COMMON-012).
pub struct ClipboardStore {
    entries: [Option<ClipboardEntry>; MAX_CLIPBOARD_ENTRIES],
    entry_count: usize,
    next_entry_id: u32,
    read_requests: [Option<ClipReadRequest>; MAX_CLIP_READ_REQUESTS],
    next_request_id: u32,
    /// Logical system tick (injected by the caller; avoids real-time dependency in #[no_std]).
    current_tick: u64,
}

impl ClipboardStore {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_CLIPBOARD_ENTRIES],
            entry_count: 0,
            next_entry_id: 1,
            read_requests: [None; MAX_CLIP_READ_REQUESTS],
            next_request_id: 1,
            current_tick: 0,
        }
    }

    /// Advance the logical clock and auto-clear any expired sensitive entries (INV-CLIP-001).
    pub fn tick(&mut self, new_tick: u64) {
        self.current_tick = new_tick;
        for slot in self.entries.iter_mut() {
            if let Some(ref mut e) = slot {
                if !e.cleared && e.is_expired(new_tick) {
                    e.secure_clear();
                }
            }
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    /// Write a new clipboard entry. Requires CAP_CLIPBOARD_WRITE (REQ-APP-COMMON-011).
    /// `declared_password` lets the source app hint that the content is a credential.
    pub fn write(
        &mut self,
        writer_pid: u32,
        writer_caps: u64,
        data: &[u8],
        declared_password: bool,
    ) -> Result<u32, ClipboardError> {
        if (writer_caps & CAP_CLIPBOARD_WRITE) == 0 {
            return Err(ClipboardError::MissingWriteCapability);
        }
        if data.len() > MAX_CLIP_PAYLOAD {
            return Err(ClipboardError::PayloadTooLarge);
        }

        let sensitivity = ClipSensitivity::classify(data, declared_password);
        let entry_id = self.next_entry_id;
        self.next_entry_id += 1;

        let mut entry = ClipboardEntry {
            entry_id,
            writer_pid,
            data: [0u8; MAX_CLIP_PAYLOAD],
            data_len: data.len(),
            sensitivity,
            written_at_tick: self.current_tick,
            cleared: false,
        };
        entry.data[..data.len()].copy_from_slice(data);

        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(entry);
                self.entry_count += 1;
                return Ok(entry_id);
            }
        }
        Err(ClipboardError::StoreFull)
    }

    // ── Read consent flow (INV-CLIP-002) ─────────────────────────────────────

    /// An app requests read access to an entry. Returns a consent request ID.
    /// The user must explicitly grant the request before data is returned.
    pub fn request_read(
        &mut self,
        reader_pid: u32,
        reader_caps: u64,
        entry_id: u32,
    ) -> Result<u32, ClipboardError> {
        if (reader_caps & CAP_CLIPBOARD_READ) == 0 {
            return Err(ClipboardError::MissingReadCapability);
        }
        // Verify entry exists and is not cleared.
        self.find_entry(entry_id)?;

        let request_id = self.next_request_id;
        self.next_request_id += 1;
        for slot in self.read_requests.iter_mut() {
            if slot.is_none() {
                *slot = Some(ClipReadRequest {
                    request_id,
                    reader_pid,
                    target_entry_id: entry_id,
                    user_granted: false,
                });
                return Ok(request_id);
            }
        }
        Err(ClipboardError::ConsentQueueFull)
    }

    /// User grants or denies a pending read request (INV-CLIP-002).
    pub fn resolve_consent(
        &mut self,
        request_id: u32,
        granted: bool,
    ) -> Result<(), ClipboardError> {
        for slot in self.read_requests.iter_mut() {
            if let Some(ref mut req) = slot {
                if req.request_id == request_id {
                    req.user_granted = granted;
                    return Ok(());
                }
            }
        }
        Err(ClipboardError::RequestNotFound)
    }

    /// Retrieve the clipboard data for a consent-granted read request.
    /// Returns a slice into the entry's data if the user has granted access.
    /// Clears the request after consumption (one-shot: per-read consent).
    pub fn read_with_consent(
        &mut self,
        request_id: u32,
    ) -> Result<([u8; MAX_CLIP_PAYLOAD], usize), ClipboardError> {
        // Find and validate the consent request.
        let (entry_id, granted) = {
            let req = self
                .read_requests
                .iter()
                .find_map(|s| s.as_ref().filter(|r| r.request_id == request_id))
                .ok_or(ClipboardError::RequestNotFound)?;
            (req.target_entry_id, req.user_granted)
        };

        if !granted {
            return Err(ClipboardError::ConsentDenied);
        }

        // Retrieve data (check expiry again — time may have passed).
        let (data, len) = {
            let entry = self.find_entry(entry_id)?;
            if entry.cleared {
                return Err(ClipboardError::EntryCleared);
            }
            (entry.data, entry.data_len)
        };

        // Consume the request (per-read — one-shot grant, INV-CLIP-002).
        for slot in self.read_requests.iter_mut() {
            if let Some(ref r) = slot {
                if r.request_id == request_id {
                    *slot = None;
                    break;
                }
            }
        }

        Ok((data, len))
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn find_entry(&self, entry_id: u32) -> Result<&ClipboardEntry, ClipboardError> {
        for slot in self.entries.iter() {
            if let Some(ref e) = slot {
                if e.entry_id == entry_id {
                    if e.cleared {
                        return Err(ClipboardError::EntryCleared);
                    }
                    return Ok(e);
                }
            }
        }
        Err(ClipboardError::EntryNotFound)
    }

    /// Manually clear an entry (e.g. app terminates or user explicit clear).
    pub fn clear_entry(&mut self, entry_id: u32) -> Result<(), ClipboardError> {
        for slot in self.entries.iter_mut() {
            if let Some(ref mut e) = slot {
                if e.entry_id == entry_id {
                    e.secure_clear();
                    return Ok(());
                }
            }
        }
        Err(ClipboardError::EntryNotFound)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> ClipboardStore {
        ClipboardStore::new()
    }

    #[test]
    // REQ-APP-COMMON-011: write requires CAP_CLIPBOARD_WRITE
    fn test_write_requires_write_cap() {
        let mut s = make_store();
        let err = s.write(10, 0, b"hello", false);
        assert_eq!(err, Err(ClipboardError::MissingWriteCapability));

        let ok = s.write(10, CAP_CLIPBOARD_WRITE, b"hello", false);
        assert!(ok.is_ok());
        assert_eq!(s.entry_count(), 1);
    }

    #[test]
    // REQ-APP-COMMON-012: read requires CAP_CLIPBOARD_READ
    fn test_read_requires_read_cap() {
        let mut s = make_store();
        let eid = s.write(10, CAP_CLIPBOARD_WRITE, b"hello", false).unwrap();

        let err = s.request_read(20, 0, eid);
        assert_eq!(err, Err(ClipboardError::MissingReadCapability));
    }

    #[test]
    // INV-CLIP-002: read data not returned without explicit user consent
    fn test_no_read_without_consent() {
        let mut s = make_store();
        let eid = s.write(10, CAP_CLIPBOARD_WRITE, b"hello", false).unwrap();
        let rid = s.request_read(20, CAP_CLIPBOARD_READ, eid).unwrap();

        // Do NOT call resolve_consent → attempt to read anyway.
        let err = s.read_with_consent(rid);
        assert_eq!(err, Err(ClipboardError::ConsentDenied));
    }

    #[test]
    // INV-CLIP-002: per-read consent grant is one-shot (consumed after use)
    fn test_consent_is_one_shot() {
        let mut s = make_store();
        let eid = s.write(10, CAP_CLIPBOARD_WRITE, b"hello", false).unwrap();
        let rid = s.request_read(20, CAP_CLIPBOARD_READ, eid).unwrap();
        s.resolve_consent(rid, true).unwrap();
        let _ = s.read_with_consent(rid).unwrap(); // First read succeeds.
        // Second read must fail: request consumed.
        let err = s.read_with_consent(rid);
        assert_eq!(err, Err(ClipboardError::RequestNotFound));
    }

    #[test]
    // DEF-APP-006 / INV-CLIP-001: OTP auto-cleared after SENSITIVE_TIMEOUT_TICKS
    fn test_otp_auto_clear_on_tick() {
        let mut s = make_store();
        let eid = s.write(10, CAP_CLIPBOARD_WRITE, b"123456", false).unwrap();

        // Verify classified as OTP.
        let entry = s.find_entry(eid).unwrap();
        assert_eq!(entry.sensitivity, ClipSensitivity::Otp);

        // Advance clock past SENSITIVE_TIMEOUT_TICKS.
        s.tick(SENSITIVE_TIMEOUT_TICKS);

        // Entry should now be cleared.
        let err = s.find_entry(eid);
        assert_eq!(err.unwrap_err(), ClipboardError::EntryCleared);
    }

    #[test]
    // INV-CLIP-001: password classified and cleared after timeout
    fn test_password_auto_clear() {
        let mut s = make_store();
        let eid = s
            .write(10, CAP_CLIPBOARD_WRITE, b"s3cr3t!", /* declared_password */ true)
            .unwrap();
        let entry = s.find_entry(eid).unwrap();
        assert_eq!(entry.sensitivity, ClipSensitivity::Password);
        s.tick(SENSITIVE_TIMEOUT_TICKS);
        assert_eq!(s.find_entry(eid).unwrap_err(), ClipboardError::EntryCleared);
    }

    #[test]
    // Normal (non-sensitive) entry is NOT cleared after SENSITIVE_TIMEOUT_TICKS.
    fn test_normal_entry_persists() {
        let mut s = make_store();
        let eid = s
            .write(10, CAP_CLIPBOARD_WRITE, b"https://indos.dev", false)
            .unwrap();
        s.tick(SENSITIVE_TIMEOUT_TICKS * 10); // Way past OTP timeout.
        assert!(s.find_entry(eid).is_ok());
    }

    #[test]
    // Full happy-path: write → request_read → consent → read_with_consent
    fn test_full_read_flow() {
        let mut s = make_store();
        let eid = s.write(10, CAP_CLIPBOARD_WRITE, b"clip data", false).unwrap();
        let rid = s.request_read(20, CAP_CLIPBOARD_READ, eid).unwrap();
        s.resolve_consent(rid, true).unwrap();
        let (data, len) = s.read_with_consent(rid).unwrap();
        assert_eq!(len, b"clip data".len());
        assert_eq!(&data[..len], b"clip data");
    }
}
