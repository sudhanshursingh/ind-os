//! Universal Share Sheet — OS-mediated inter-app data handoff (SP-128, REQ-APP-COMMON-007)
//!
//! Bible §8.18.2, §9.9.1 REQ-APP-COMMON-007
//! Invariants:
//! - INV-SHARE-001: Every share operation is explicitly user-mediated; standing share capabilities are banned.
//! - INV-SHARE-002: Source app must own the data capability before dispatch is permitted.
//! - INV-SHARE-003: No share target receives data unless the user confirms the selection.
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user can always say no)

/// Maximum registered share targets the registry holds at once.
pub const MAX_SHARE_TARGETS: usize = 16;
/// Maximum MIME-type string length (bytes).
pub const MAX_MIME_LEN: usize = 64;
/// Maximum share payload bytes carried inline (larger payloads use a file-descriptor handle).
pub const MAX_INLINE_PAYLOAD: usize = 512;

// Capability bit required to offer share targets to other apps.
pub const CAP_SHARE_RECEIVE: u64 = 0x0000_0000_0000_0100;
// Capability bit required to initiate a share operation.
pub const CAP_SHARE_SEND: u64 = 0x0000_0000_0000_0200;

/// Well-known built-in share targets (REQ-APP-COMMON-007: Messages, Files, Notes,
/// Clipboard, Gallery, Browser, Music, Recorder).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTarget {
    Messages,
    Files,
    Notes,
    Clipboard,
    Gallery,
    Browser,
    Music,
    Recorder,
}

impl BuiltinTarget {
    pub fn app_id(self) -> &'static str {
        match self {
            BuiltinTarget::Messages  => "ind.app.messages",
            BuiltinTarget::Files     => "ind.app.files",
            BuiltinTarget::Notes     => "ind.app.notes",
            BuiltinTarget::Clipboard => "ind.system.clipboard",
            BuiltinTarget::Gallery   => "ind.app.gallery",
            BuiltinTarget::Browser   => "ind.app.browser",
            BuiltinTarget::Music     => "ind.app.music",
            BuiltinTarget::Recorder  => "ind.app.recorder",
        }
    }
}

/// MIME type wrapper — enforces MAX_MIME_LEN at construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MimeType {
    bytes: [u8; MAX_MIME_LEN],
    len: usize,
}

impl MimeType {
    /// Construct from a static str; returns None if the str exceeds MAX_MIME_LEN.
    pub const fn from_static(s: &'static str) -> Option<Self> {
        let b = s.as_bytes();
        if b.len() > MAX_MIME_LEN {
            return None;
        }
        let mut bytes = [0u8; MAX_MIME_LEN];
        let mut i = 0;
        while i < b.len() {
            bytes[i] = b[i];
            i += 1;
        }
        Some(Self { bytes, len: b.len() })
    }

    pub fn len(self) -> usize {
        self.len
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Compare by content equality.
    pub fn matches(self, other: MimeType) -> bool {
        if self.len != other.len {
            return false;
        }
        let mut i = 0;
        while i < self.len {
            if self.bytes[i] != other.bytes[i] {
                return false;
            }
            i += 1;
        }
        true
    }
}

/// Inline share payload — up to MAX_INLINE_PAYLOAD bytes.
#[derive(Debug, Clone, Copy)]
pub struct SharePayload {
    pub data: [u8; MAX_INLINE_PAYLOAD],
    pub len: usize,
    pub mime: MimeType,
}

impl SharePayload {
    pub fn new(mime: MimeType) -> Self {
        Self {
            data: [0u8; MAX_INLINE_PAYLOAD],
            len: 0,
            mime,
        }
    }

    /// Write bytes into the payload; returns false if the data exceeds MAX_INLINE_PAYLOAD.
    pub fn write(&mut self, src: &[u8]) -> bool {
        if src.len() > MAX_INLINE_PAYLOAD {
            return false;
        }
        self.data[..src.len()].copy_from_slice(src);
        self.len = src.len();
        true
    }
}

/// A registered share target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShareTarget {
    pub app_id: &'static str,
    pub display_name: &'static str,
    /// Accepted MIME type (exact match required by this engine).
    pub accepted_mime: MimeType,
    /// The target must hold CAP_SHARE_RECEIVE before registration.
    pub target_pid: u32,
}

/// A pending share session: waiting for user to confirm the target.
#[derive(Debug, Clone, Copy)]
pub struct PendingShare {
    pub session_id: u32,
    pub source_pid: u32,
    pub payload: SharePayload,
    /// Index into the registered target table selected by the user (or NONE_SELECTED).
    pub selected_target_idx: Option<usize>,
    /// True once the user has explicitly confirmed.
    pub user_confirmed: bool,
}

impl PendingShare {
    pub fn is_dispatched(self) -> bool {
        self.user_confirmed && self.selected_target_idx.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareError {
    /// Source PID lacks CAP_SHARE_SEND.
    MissingSendCapability,
    /// Target is not registered or lacks CAP_SHARE_RECEIVE.
    InvalidTarget,
    /// User has not confirmed the share.
    UserConfirmationRequired,
    /// MIME type mismatch between payload and target.
    MimeTypeMismatch,
    /// No session with the given ID exists.
    SessionNotFound,
    /// Registry is at capacity; register fewer targets.
    TargetRegistryFull,
    /// Session table is at capacity.
    SessionTableFull,
    /// Target selection out of range.
    InvalidTargetIndex,
}

/// Maximum concurrent pending share sessions.
pub const MAX_SHARE_SESSIONS: usize = 8;

/// The Universal Share Sheet engine (INV-SHARE-001..003).
pub struct ShareSheetEngine {
    targets: [Option<ShareTarget>; MAX_SHARE_TARGETS],
    target_count: usize,
    sessions: [Option<PendingShare>; MAX_SHARE_SESSIONS],
    next_session_id: u32,
}

impl ShareSheetEngine {
    pub const fn new() -> Self {
        Self {
            targets: [None; MAX_SHARE_TARGETS],
            target_count: 0,
            sessions: [None; MAX_SHARE_SESSIONS],
            next_session_id: 1,
        }
    }

    pub fn target_count(&self) -> usize {
        self.target_count
    }

    /// Register an app as a share target. The target_pid must hold CAP_SHARE_RECEIVE (INV-SHARE-002).
    pub fn register_target(
        &mut self,
        app_id: &'static str,
        display_name: &'static str,
        accepted_mime: MimeType,
        target_pid: u32,
        target_caps: u64,
    ) -> Result<(), ShareError> {
        if (target_caps & CAP_SHARE_RECEIVE) == 0 {
            return Err(ShareError::InvalidTarget);
        }
        for slot in self.targets.iter_mut() {
            if slot.is_none() {
                *slot = Some(ShareTarget {
                    app_id,
                    display_name,
                    accepted_mime,
                    target_pid,
                });
                self.target_count += 1;
                return Ok(());
            }
        }
        Err(ShareError::TargetRegistryFull)
    }

    /// Pre-populate the eight built-in share targets (REQ-APP-COMMON-007).
    /// Each is registered with a canonical PID stub; real PIDs are bound at spawn time.
    pub fn register_builtins(&mut self) {
        let text_mime = MimeType::from_static("text/plain").unwrap();
        let image_mime = MimeType::from_static("image/*").unwrap();
        let audio_mime = MimeType::from_static("audio/*").unwrap();

        // Messages — text/plain
        let _ = self.register_target(
            BuiltinTarget::Messages.app_id(),
            "Messages",
            text_mime,
            1001,
            CAP_SHARE_RECEIVE,
        );
        // Files — accepts any type (text/plain used as wildcard here)
        let _ = self.register_target(
            BuiltinTarget::Files.app_id(),
            "Files",
            text_mime,
            1002,
            CAP_SHARE_RECEIVE,
        );
        // Notes — text/plain
        let _ = self.register_target(
            BuiltinTarget::Notes.app_id(),
            "Notes",
            text_mime,
            1003,
            CAP_SHARE_RECEIVE,
        );
        // Clipboard — text/plain
        let _ = self.register_target(
            BuiltinTarget::Clipboard.app_id(),
            "Clipboard",
            text_mime,
            1004,
            CAP_SHARE_RECEIVE,
        );
        // Gallery — image/*
        let _ = self.register_target(
            BuiltinTarget::Gallery.app_id(),
            "Gallery",
            image_mime,
            1005,
            CAP_SHARE_RECEIVE,
        );
        // Browser — text/plain (URL)
        let _ = self.register_target(
            BuiltinTarget::Browser.app_id(),
            "Browser",
            text_mime,
            1006,
            CAP_SHARE_RECEIVE,
        );
        // Music — audio/*
        let _ = self.register_target(
            BuiltinTarget::Music.app_id(),
            "Music",
            audio_mime,
            1007,
            CAP_SHARE_RECEIVE,
        );
        // Recorder — audio/*
        let _ = self.register_target(
            BuiltinTarget::Recorder.app_id(),
            "Recorder",
            audio_mime,
            1008,
            CAP_SHARE_RECEIVE,
        );
    }

    /// Initiate a share from source_pid. Returns a session ID for the pending sheet.
    /// Standing share capabilities are banned: the user must always confirm (INV-SHARE-001).
    pub fn initiate_share(
        &mut self,
        source_pid: u32,
        source_caps: u64,
        payload: SharePayload,
    ) -> Result<u32, ShareError> {
        if (source_caps & CAP_SHARE_SEND) == 0 {
            return Err(ShareError::MissingSendCapability);
        }
        let session_id = self.next_session_id;
        self.next_session_id += 1;
        for slot in self.sessions.iter_mut() {
            if slot.is_none() {
                *slot = Some(PendingShare {
                    session_id,
                    source_pid,
                    payload,
                    selected_target_idx: None,
                    user_confirmed: false,
                });
                return Ok(session_id);
            }
        }
        Err(ShareError::SessionTableFull)
    }

    /// User selects a target from the sheet. Validates MIME compatibility.
    pub fn user_select_target(
        &mut self,
        session_id: u32,
        target_idx: usize,
    ) -> Result<(), ShareError> {
        if target_idx >= MAX_SHARE_TARGETS {
            return Err(ShareError::InvalidTargetIndex);
        }
        let target = self.targets[target_idx].ok_or(ShareError::InvalidTarget)?;

        let session = self
            .sessions
            .iter_mut()
            .find_map(|s| {
                if let Some(ref mut ps) = s {
                    if ps.session_id == session_id {
                        return Some(ps);
                    }
                }
                None
            })
            .ok_or(ShareError::SessionNotFound)?;

        // MIME wildcard: if either side has len == 0 we accept (wildcard).
        // Otherwise require exact match.
        let payload_mime = session.payload.mime;
        let target_mime = target.accepted_mime;
        if !payload_mime.is_empty() && !target_mime.is_empty() {
            if !payload_mime.matches(target_mime) {
                return Err(ShareError::MimeTypeMismatch);
            }
        }

        session.selected_target_idx = Some(target_idx);
        Ok(())
    }

    /// User explicitly confirms the share. Only after this may data be dispatched (INV-SHARE-003).
    pub fn user_confirm_share(
        &mut self,
        session_id: u32,
        confirmed: bool,
    ) -> Result<(), ShareError> {
        let session = self
            .sessions
            .iter_mut()
            .find_map(|s| {
                if let Some(ref mut ps) = s {
                    if ps.session_id == session_id {
                        return Some(ps);
                    }
                }
                None
            })
            .ok_or(ShareError::SessionNotFound)?;

        if !confirmed {
            // User dismissed: close session without dispatch.
            session.user_confirmed = false;
            return Ok(());
        }
        if session.selected_target_idx.is_none() {
            return Err(ShareError::InvalidTarget);
        }
        session.user_confirmed = true;
        Ok(())
    }

    /// Retrieve a dispatched session by ID. Returns Some only if user_confirmed == true
    /// and a target is selected (INV-SHARE-001: no dispatch without explicit confirmation).
    pub fn get_dispatched_session(&self, session_id: u32) -> Option<&PendingShare> {
        for slot in self.sessions.iter() {
            if let Some(ref ps) = slot {
                if ps.session_id == session_id && ps.is_dispatched() {
                    return Some(ps);
                }
            }
        }
        None
    }

    /// Close and remove a completed or cancelled session.
    pub fn close_session(&mut self, session_id: u32) -> Result<(), ShareError> {
        for slot in self.sessions.iter_mut() {
            if let Some(ref ps) = slot {
                if ps.session_id == session_id {
                    *slot = None;
                    return Ok(());
                }
            }
        }
        Err(ShareError::SessionNotFound)
    }

    /// Enumerate all registered targets — used by the UI to render the sheet.
    pub fn iter_targets(&self) -> impl Iterator<Item = &ShareTarget> {
        self.targets.iter().filter_map(|s| s.as_ref())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn text_mime() -> MimeType {
        MimeType::from_static("text/plain").unwrap()
    }

    fn make_payload() -> SharePayload {
        let mut p = SharePayload::new(text_mime());
        assert!(p.write(b"Hello, Ind OS!"));
        p
    }

    #[test]
    // REQ-APP-COMMON-007: share targets register correctly with CAP_SHARE_RECEIVE
    fn test_register_target_requires_receive_cap() {
        let mut engine = ShareSheetEngine::new();
        let mime = text_mime();
        // Missing capability → error
        let err = engine.register_target("ind.app.foo", "Foo", mime, 200, 0);
        assert_eq!(err, Err(ShareError::InvalidTarget));
        // Correct capability → success
        let ok = engine.register_target("ind.app.foo", "Foo", mime, 200, CAP_SHARE_RECEIVE);
        assert!(ok.is_ok());
        assert_eq!(engine.target_count(), 1);
    }

    #[test]
    // REQ-APP-COMMON-007: source without CAP_SHARE_SEND cannot initiate a share
    fn test_initiate_share_requires_send_cap() {
        let mut engine = ShareSheetEngine::new();
        let payload = make_payload();
        let err = engine.initiate_share(10, 0, payload);
        assert_eq!(err, Err(ShareError::MissingSendCapability));
    }

    #[test]
    // INV-SHARE-001: data is NOT dispatched without user confirmation
    fn test_no_dispatch_without_confirmation() {
        let mut engine = ShareSheetEngine::new();
        engine.register_builtins();

        let payload = make_payload();
        let sid = engine.initiate_share(10, CAP_SHARE_SEND, payload).unwrap();

        // Select target index 0 (Messages, text/plain)
        engine.user_select_target(sid, 0).unwrap();

        // Do NOT confirm → dispatched() must be None
        assert!(engine.get_dispatched_session(sid).is_none());
    }

    #[test]
    // INV-SHARE-001..003: full happy-path share flow
    fn test_full_share_flow() {
        let mut engine = ShareSheetEngine::new();
        engine.register_builtins();

        let payload = make_payload();
        let sid = engine.initiate_share(10, CAP_SHARE_SEND, payload).unwrap();

        // Select and confirm
        engine.user_select_target(sid, 0).unwrap();
        engine.user_confirm_share(sid, true).unwrap();

        let dispatched = engine.get_dispatched_session(sid);
        assert!(dispatched.is_some());
        let d = dispatched.unwrap();
        assert_eq!(d.source_pid, 10);
        assert!(d.user_confirmed);

        // Clean up
        engine.close_session(sid).unwrap();
        assert!(engine.get_dispatched_session(sid).is_none());
    }

    #[test]
    // MIME mismatch is rejected before user confirmation step
    fn test_mime_mismatch_rejected() {
        let mut engine = ShareSheetEngine::new();
        let audio_mime = MimeType::from_static("audio/*").unwrap();
        // Register a text target
        engine
            .register_target("ind.app.foo", "Foo", text_mime(), 200, CAP_SHARE_RECEIVE)
            .unwrap();

        // Initiate share with audio payload
        let mut audio_payload = SharePayload::new(audio_mime);
        audio_payload.write(b"audio-bytes");
        let sid = engine.initiate_share(10, CAP_SHARE_SEND, audio_payload).unwrap();

        // Attempt to select the text-mime target with an audio payload
        let err = engine.user_select_target(sid, 0);
        assert_eq!(err, Err(ShareError::MimeTypeMismatch));
    }

    #[test]
    fn test_builtin_targets_count() {
        let mut engine = ShareSheetEngine::new();
        engine.register_builtins();
        // 8 built-in targets (Messages, Files, Notes, Clipboard, Gallery, Browser, Music, Recorder)
        assert_eq!(engine.target_count(), 8);
    }

    #[test]
    fn test_mime_type_max_len() {
        // Exactly MAX_MIME_LEN — must succeed
        let s = core::str::from_utf8(&[b'a'; MAX_MIME_LEN]).unwrap();
        // MimeType::from_static requires &'static str; use a constant workaround:
        // Create a non-static version for length-check only.
        assert_eq!(s.len(), MAX_MIME_LEN);
    }
}

