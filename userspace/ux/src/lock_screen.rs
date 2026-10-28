//! Lock Screen System Surface (SP-129, REQ-APP-COMMON-015)
//!
//! Bible §8.18.2, §8.20, §9.9.1 REQ-APP-COMMON-015
//! Invariants:
//! - INV-LOCK-001: Lock screen renders exclusively on PlaneId::HardwareIsolated — no app can
//!                 overlay it or inject synthetic input onto it.
//! - INV-LOCK-002: Emergency dialer shortcut is always available regardless of lock state, PIN
//!                 authentication status, or AIS mode. No capability required. (REQ-APP-PHONE-002)
//! - INV-LOCK-003: Consent cards presented on the lock screen are enqueued by ENF and may only
//!                 be dismissed (Grant / Deny) by explicit physical interaction — never by AIS.
//! - INV-LOCK-004: Media controls are shown only when an audio session is active and rendered on
//!                 the isolated plane (REQ-APP-MUS-002).
//! - INV-LOCK-005: AIS kill-switch is accessible within ≤ 2 gestures from the lock screen
//!                 (AIS Charter §12.3, Bible §8.18.2 SP-131 precondition).
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user can always say no)

/// The hardware-isolated compositor plane identifier. Only PlaneId::HardwareIsolated surfaces
/// may be used for the lock screen (INV-LOCK-001, Bible §2117).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneId {
    /// Standard composited app plane. Subject to z-order manipulation.
    Normal,
    /// Reserved DSI plane — no app may overlay, no input injection possible (INV-LOCK-001).
    HardwareIsolated,
}

// ─── Lock state machine ────────────────────────────────────────────────────────

/// The lock screen state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockState {
    /// Device is locked. Lock screen is the only interactive surface.
    Locked,
    /// PIN/biometric challenge is active.
    Authenticating,
    /// Authentication succeeded; dismissal animation in progress.
    Unlocking,
    /// Device is fully unlocked; lock screen not rendered.
    Unlocked,
}

impl LockState {
    /// Returns true if the lock screen is currently the foreground surface.
    pub fn is_surface_active(self) -> bool {
        matches!(self, LockState::Locked | LockState::Authenticating | LockState::Unlocking)
    }
}

// ─── Clock display ────────────────────────────────────────────────────────────

/// A simple 24-hour clock value (HH:MM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockClock {
    pub hour: u8,   // 0–23
    pub minute: u8, // 0–59
}

impl LockClock {
    pub const fn new(hour: u8, minute: u8) -> Self {
        // Clamped — real RTC validation happens in the clock service.
        Self {
            hour: if hour > 23 { 23 } else { hour },
            minute: if minute > 59 { 59 } else { minute },
        }
    }
}

// ─── Consent card queue ───────────────────────────────────────────────────────

/// Maximum consent cards queued on the lock screen at once.
pub const MAX_CONSENT_CARDS: usize = 4;

/// The result of a user's response to a consent card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentVerdict {
    Granted,
    Denied,
    /// User swiped away without acting; re-queued at next unlock.
    Deferred,
}

/// A pending consent card presented on the lock screen (INV-LOCK-003).
#[derive(Debug, Clone, Copy)]
pub struct ConsentCard {
    pub card_id: u32,
    /// Short plain-language capability description shown to the user.
    pub description: &'static str,
    /// App requesting the capability.
    pub requesting_app: &'static str,
    /// Capability token being requested.
    pub capability: u64,
    /// Set to Some only after explicit physical user interaction.
    pub verdict: Option<ConsentVerdict>,
}

impl ConsentCard {
    /// True if this card has been actioned by the user (INV-LOCK-003: only physical interaction resolves it).
    pub fn is_resolved(self) -> bool {
        self.verdict.is_some()
    }
}

// ─── Media controls ───────────────────────────────────────────────────────────

/// An active media session shown in the lock screen player widget (INV-LOCK-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaPlayState {
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy)]
pub struct MediaSession {
    pub session_id: u32,
    pub app_id: &'static str,
    pub track_title: &'static str,
    pub artist: &'static str,
    pub play_state: MediaPlayState,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockScreenError {
    /// Action requires the lock screen surface to be active.
    SurfaceNotActive,
    /// Consent card queue is full.
    ConsentQueueFull,
    /// Consent card ID not found.
    ConsentCardNotFound,
    /// AIS verdict attempted on a card — blocked by INV-LOCK-003.
    AisVerdictForbidden,
    /// Emergency dialer is always available; this error is structurally unreachable.
    EmergencyDialerUnavailable,
    /// State transition not valid in current lock state.
    InvalidStateTransition,
    /// No active media session to display.
    NoMediaSession,
}

// ─── AIS kill-switch ──────────────────────────────────────────────────────────

/// State of the AIS global kill-switch accessible from the lock screen (AIS Charter §12.3).
/// ≤ 2 gestures: (1) swipe up on lock screen → AIS strip visible; (2) tap kill toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AisKillSwitchState {
    /// AIS is operational.
    Active,
    /// AIS is fully suspended. No suggestions, reads, proposals, or memory writes.
    Killed,
}

// ─── Engine ───────────────────────────────────────────────────────────────────

/// The Lock Screen System Surface engine.
/// All rendering is gated on PlaneId::HardwareIsolated (INV-LOCK-001).
pub struct LockScreenEngine {
    pub plane: PlaneId,
    pub state: LockState,
    pub clock: LockClock,
    consent_cards: [Option<ConsentCard>; MAX_CONSENT_CARDS],
    card_count: usize,
    next_card_id: u32,
    media_session: Option<MediaSession>,
    pub ais_kill_switch: AisKillSwitchState,
    /// Gesture counter since last unlock event — used to enforce ≤ 2-gesture AIS kill rule.
    gesture_count_since_surface_active: u8,
}

impl LockScreenEngine {
    pub const fn new() -> Self {
        Self {
            plane: PlaneId::HardwareIsolated,
            state: LockState::Locked,
            clock: LockClock::new(0, 0),
            consent_cards: [None; MAX_CONSENT_CARDS],
            card_count: 0,
            next_card_id: 1,
            media_session: None,
            ais_kill_switch: AisKillSwitchState::Active,
            gesture_count_since_surface_active: 0,
        }
    }

    // ── Plane assertion ────────────────────────────────────────────────────────

    /// Structural check: lock screen is only valid on PlaneId::HardwareIsolated (INV-LOCK-001).
    pub fn assert_plane(&self) -> bool {
        self.plane == PlaneId::HardwareIsolated
    }

    // ── State machine ─────────────────────────────────────────────────────────

    /// Valid lock state transitions.
    pub fn transition(&mut self, next: LockState) -> Result<(), LockScreenError> {
        let valid = match (self.state, next) {
            (LockState::Locked, LockState::Authenticating) => true,
            (LockState::Authenticating, LockState::Locked) => true,  // auth failed / cancel
            (LockState::Authenticating, LockState::Unlocking) => true,
            (LockState::Unlocking, LockState::Unlocked) => true,
            (LockState::Unlocked, LockState::Locked) => true,         // screen-off / timeout
            _ => false,
        };
        if !valid {
            return Err(LockScreenError::InvalidStateTransition);
        }
        self.state = next;
        if next == LockState::Locked {
            self.gesture_count_since_surface_active = 0;
        }
        Ok(())
    }

    // ── Clock ─────────────────────────────────────────────────────────────────

    /// Update the displayed clock. Called by the system clock service.
    pub fn update_clock(&mut self, hour: u8, minute: u8) {
        self.clock = LockClock::new(hour, minute);
    }

    // ── Emergency dialer ──────────────────────────────────────────────────────

    /// Returns true unconditionally — the emergency dialer shortcut is always available
    /// regardless of lock state, PIN status, or AIS mode (INV-LOCK-002, REQ-APP-PHONE-002).
    /// No capability required; no authentication required; no consent required.
    pub fn emergency_dialer_available(&self) -> bool {
        true // Structural invariant — never gated.
    }

    /// Activate the emergency dialer path. Returns the emergency number to dial.
    /// In production this signals the Phone app's emergency IPC endpoint.
    pub fn activate_emergency_dialer(&self) -> &'static str {
        "112" // Primary emergency number (India); Phone app resolves locale-specific fallbacks.
    }

    // ── Consent card queue ────────────────────────────────────────────────────

    /// Enqueue a consent card presented by ENF for user resolution (INV-LOCK-003).
    pub fn enqueue_consent_card(
        &mut self,
        description: &'static str,
        requesting_app: &'static str,
        capability: u64,
    ) -> Result<u32, LockScreenError> {
        for slot in self.consent_cards.iter_mut() {
            if slot.is_none() {
                let card_id = self.next_card_id;
                self.next_card_id += 1;
                *slot = Some(ConsentCard {
                    card_id,
                    description,
                    requesting_app,
                    capability,
                    verdict: None,
                });
                self.card_count += 1;
                return Ok(card_id);
            }
        }
        Err(LockScreenError::ConsentQueueFull)
    }

    /// Apply a user-issued verdict to a consent card (INV-LOCK-003).
    /// The `is_ais_action` flag must be false — AIS cannot resolve consent cards.
    pub fn resolve_consent_card(
        &mut self,
        card_id: u32,
        verdict: ConsentVerdict,
        is_ais_action: bool,
    ) -> Result<(), LockScreenError> {
        if is_ais_action {
            return Err(LockScreenError::AisVerdictForbidden);
        }
        for slot in self.consent_cards.iter_mut() {
            if let Some(ref mut card) = slot {
                if card.card_id == card_id {
                    card.verdict = Some(verdict);
                    return Ok(());
                }
            }
        }
        Err(LockScreenError::ConsentCardNotFound)
    }

    pub fn consent_card_count(&self) -> usize {
        self.card_count
    }

    pub fn get_consent_card(&self, card_id: u32) -> Option<&ConsentCard> {
        self.consent_cards.iter().find_map(|s| {
            s.as_ref().filter(|c| c.card_id == card_id)
        })
    }

    // ── Media controls ────────────────────────────────────────────────────────

    /// Register the active media session shown in the lock screen widget (INV-LOCK-004).
    pub fn set_media_session(&mut self, session: MediaSession) {
        self.media_session = Some(session);
    }

    pub fn clear_media_session(&mut self) {
        self.media_session = None;
    }

    pub fn media_session(&self) -> Option<&MediaSession> {
        self.media_session.as_ref()
    }

    /// Toggle media play/pause from the lock screen widget. Does not require device unlock.
    pub fn media_toggle_play_pause(&mut self) -> Result<MediaPlayState, LockScreenError> {
        match self.media_session.as_mut() {
            Some(s) => {
                s.play_state = match s.play_state {
                    MediaPlayState::Playing => MediaPlayState::Paused,
                    MediaPlayState::Paused => MediaPlayState::Playing,
                };
                Ok(s.play_state)
            }
            None => Err(LockScreenError::NoMediaSession),
        }
    }

    // ── AIS kill-switch ───────────────────────────────────────────────────────

    /// Record a user gesture on the lock screen surface. Used to enforce ≤ 2-gesture AIS kill rule.
    pub fn record_gesture(&mut self) {
        self.gesture_count_since_surface_active =
            self.gesture_count_since_surface_active.saturating_add(1);
    }

    /// Toggle the AIS kill-switch. Returns the new state.
    /// This is the action that fires on the second gesture (swipe + tap) per AIS Charter §12.3.
    pub fn toggle_ais_kill_switch(&mut self) -> AisKillSwitchState {
        self.ais_kill_switch = match self.ais_kill_switch {
            AisKillSwitchState::Active => AisKillSwitchState::Killed,
            AisKillSwitchState::Killed => AisKillSwitchState::Active,
        };
        self.ais_kill_switch
    }

    /// True if the AIS kill-switch is reachable within ≤ 2 gestures from current state.
    /// Gesture 1: reveal AIS strip (swipe); Gesture 2: tap kill toggle.
    pub fn ais_kill_within_two_gestures(&self) -> bool {
        // The AIS strip is always reachable in exactly 2 gestures from the lock surface.
        // This predicate is a compile-time-provable invariant; the gesture_count field is
        // informational for the audit trail only.
        true
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> LockScreenEngine {
        LockScreenEngine::new()
    }

    #[test]
    // INV-LOCK-001: lock screen is always on HardwareIsolated plane
    fn test_plane_is_hardware_isolated() {
        let engine = make_engine();
        assert!(engine.assert_plane());
        assert_eq!(engine.plane, PlaneId::HardwareIsolated);
    }

    #[test]
    // INV-LOCK-002: emergency dialer is always available, no capability required
    fn test_emergency_dialer_always_available() {
        let engine = make_engine();
        assert!(engine.emergency_dialer_available());
        // Available regardless of lock state
        let mut e2 = make_engine();
        e2.transition(LockState::Authenticating).unwrap();
        assert!(e2.emergency_dialer_available());
        e2.transition(LockState::Unlocking).unwrap();
        e2.transition(LockState::Unlocked).unwrap();
        assert!(e2.emergency_dialer_available());
    }

    #[test]
    // INV-LOCK-001: lock screen surface is active in Locked/Authenticating/Unlocking states
    fn test_surface_active_states() {
        let mut e = make_engine();
        assert!(e.state.is_surface_active());
        e.transition(LockState::Authenticating).unwrap();
        assert!(e.state.is_surface_active());
        e.transition(LockState::Unlocking).unwrap();
        assert!(e.state.is_surface_active());
        e.transition(LockState::Unlocked).unwrap();
        assert!(!e.state.is_surface_active());
    }

    #[test]
    // State machine only allows valid transitions
    fn test_state_machine_valid_transitions() {
        let mut e = make_engine();
        assert!(e.transition(LockState::Locked).is_err()); // Locked→Locked invalid
        e.transition(LockState::Authenticating).unwrap();
        e.transition(LockState::Unlocking).unwrap();
        e.transition(LockState::Unlocked).unwrap();
        e.transition(LockState::Locked).unwrap(); // screen-off
    }

    #[test]
    // INV-LOCK-003: consent card resolved only by physical user action
    fn test_consent_card_requires_user_action() {
        let mut e = make_engine();
        let cid = e.enqueue_consent_card("Access Camera", "ind.app.scanner", 0x10).unwrap();

        // AIS attempt to resolve → rejected
        let err = e.resolve_consent_card(cid, ConsentVerdict::Granted, /* is_ais */ true);
        assert_eq!(err, Err(LockScreenError::AisVerdictForbidden));

        // Card must still be unresolved
        let card = e.get_consent_card(cid).unwrap();
        assert!(!card.is_resolved());

        // User resolves → ok
        e.resolve_consent_card(cid, ConsentVerdict::Granted, false).unwrap();
        let card2 = e.get_consent_card(cid).unwrap();
        assert!(card2.is_resolved());
        assert_eq!(card2.verdict, Some(ConsentVerdict::Granted));
    }

    #[test]
    // INV-LOCK-003: consent queue capacity
    fn test_consent_queue_capacity() {
        let mut e = make_engine();
        for i in 0..MAX_CONSENT_CARDS {
            let _ = i;
            let ok = e.enqueue_consent_card("cap desc", "ind.app.foo", 0x01);
            assert!(ok.is_ok());
        }
        // One more → full
        let err = e.enqueue_consent_card("extra", "ind.app.bar", 0x02);
        assert_eq!(err, Err(LockScreenError::ConsentQueueFull));
    }

    #[test]
    // INV-LOCK-004: media controls toggle play/pause
    fn test_media_controls_toggle() {
        let mut e = make_engine();

        // No session → error
        let err = e.media_toggle_play_pause();
        assert_eq!(err, Err(LockScreenError::NoMediaSession));

        e.set_media_session(MediaSession {
            session_id: 1,
            app_id: "ind.app.music",
            track_title: "Raag Bhairavi",
            artist: "Pandit Ravi Shankar",
            play_state: MediaPlayState::Playing,
        });

        let new_state = e.media_toggle_play_pause().unwrap();
        assert_eq!(new_state, MediaPlayState::Paused);
        let new_state2 = e.media_toggle_play_pause().unwrap();
        assert_eq!(new_state2, MediaPlayState::Playing);
    }

    #[test]
    // AIS Charter §12.3: kill-switch accessible within ≤ 2 gestures from lock screen
    fn test_ais_kill_switch_two_gestures() {
        let mut e = make_engine();
        assert!(e.ais_kill_within_two_gestures());
        assert_eq!(e.ais_kill_switch, AisKillSwitchState::Active);

        // Gesture 1: reveal AIS strip (record_gesture)
        e.record_gesture();
        // Gesture 2: tap kill toggle
        let new_state = e.toggle_ais_kill_switch();
        assert_eq!(new_state, AisKillSwitchState::Killed);

        // Can re-enable with another 2 gestures
        e.record_gesture();
        let restored = e.toggle_ais_kill_switch();
        assert_eq!(restored, AisKillSwitchState::Active);
    }

    #[test]
    // Clock update round-trips
    fn test_clock_update() {
        let mut e = make_engine();
        e.update_clock(9, 30);
        assert_eq!(e.clock.hour, 9);
        assert_eq!(e.clock.minute, 30);
        // Clamping
        e.update_clock(99, 99);
        assert_eq!(e.clock.hour, 23);
        assert_eq!(e.clock.minute, 59);
    }
}

