//! Phone App I: Core Cellular Voice & Emergency Call Engine (SP-132, REQ-APP-PHONE-001/002/004)
//!
//! Bible §8.18.2, §8.20, §9.9.2 REQ-APP-PHONE-001, REQ-APP-PHONE-002, REQ-APP-PHONE-004
//! Invariants:
//! - INV-PHONE-001: Emergency dialer is operational unconditionally across lock states with ZERO
//!                  capabilities required; emergency numbers preempt all normal calls.
//! - INV-PHONE-002: In-call UI is rendered exclusively on PlaneId::HardwareIsolated to prevent tapjacking.
//! - INV-PHONE-003: Non-emergency cellular voice call origination requires CAP_APP_PHONE_CALLS.
//! - INV-PHONE-004: Active call acquires exclusive primary audio focus, preempting media playback.
//! - INV-PHONE-005: In-call controls (mute, speaker, hold, DTMF, end) operate deterministically with
//!                  zero heap allocation under REQ-PERF-401 budgets (<= 24 MB RAM, <= 32 ms latency).
//! Rules: N-01 (zero external crates), N-04 (no unsafe), N-05 (user authority supreme)

use crate::lock_screen::PlaneId;

/// Capability required to place or receive standard cellular voice calls (REQ-APP-PHONE-001).
pub const CAP_APP_PHONE_CALLS: u64 = 0x0040_0000_0000;

/// Maximum length of a dialed phone number or emergency string (bytes).
pub const MAX_PHONE_NUMBER_LEN: usize = 32;
/// Maximum DTMF buffer capacity.
pub const MAX_DTMF_BUFFER: usize = 32;

/// Recognized emergency numbers (REQ-APP-PHONE-002).
pub const EMERGENCY_NUMBERS: [&'static str; 6] = [
    "112", // Universal / India / EU
    "911", // US / Canada
    "100", // Police (India)
    "101", // Fire (India)
    "102", // Ambulance (India)
    "999", // UK
];

/// Strict call state machine (REQ-APP-PHONE-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallState {
    Idle,
    Dialing,
    RingingIncoming,
    RingingOutgoing,
    Active,
    Holding,
    Terminated,
    EmergencyActive,
}

impl CallState {
    pub fn is_in_call(self) -> bool {
        matches!(
            self,
            CallState::Dialing | CallState::Active | CallState::Holding | CallState::EmergencyActive
        )
    }

    pub fn is_ringing(self) -> bool {
        matches!(self, CallState::RingingIncoming | CallState::RingingOutgoing)
    }
}

/// In-call audio routing destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioRoute {
    Earpiece,
    Speakerphone,
    WiredHeadset,
    BluetoothSco,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoneCallError {
    MissingCallCapability,
    CallAlreadyActive,
    NoActiveCall,
    InvalidEmergencyNumber,
    NumberTooLong,
    InvalidStateTransition,
    PlaneNotIsolated,
    DtmfBufferFull,
}

/// In-call controls and session descriptor (INV-PHONE-002, INV-PHONE-005).
#[derive(Debug, Clone, Copy)]
pub struct InCallSession {
    pub call_id: u32,
    pub number: [u8; MAX_PHONE_NUMBER_LEN],
    pub number_len: usize,
    pub is_emergency: bool,
    pub state: CallState,
    pub audio_route: AudioRoute,
    pub is_muted: bool,
    pub is_held: bool,
    pub dtmf_digits: [u8; MAX_DTMF_BUFFER],
    pub dtmf_count: usize,
    pub duration_seconds: u32,
    pub plane: PlaneId,
}

impl InCallSession {
    pub fn number_as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.number[..self.number_len])
    }
}

/// Core Voice & Emergency Call Engine.
pub struct PhoneCallEngine {
    current_session: Option<InCallSession>,
    next_call_id: u32,
    earpiece_proximity_near: bool,
}

impl PhoneCallEngine {
    pub const fn new() -> Self {
        Self {
            current_session: None,
            next_call_id: 1,
            earpiece_proximity_near: false,
        }
    }

    pub fn current_session(&self) -> Option<&InCallSession> {
        self.current_session.as_ref()
    }

    pub fn is_call_active(&self) -> bool {
        self.current_session
            .as_ref()
            .map(|s| s.state.is_in_call() || s.state.is_ringing())
            .unwrap_or(false)
    }

    /// Returns true if an active call preempts background media playback (INV-PHONE-004).
    pub fn preempts_media_playback(&self) -> bool {
        self.is_call_active()
    }

    // ── Emergency Call Dialer (INV-PHONE-001, REQ-APP-PHONE-002) ──────────────

    /// Checks if a string is a certified emergency number.
    pub fn is_emergency_number(number: &[u8]) -> bool {
        let Ok(s) = core::str::from_utf8(number) else {
            return false;
        };
        for emergency in EMERGENCY_NUMBERS.iter() {
            if *emergency == s {
                return true;
            }
        }
        false
    }

    /// Places an emergency call. Operates unconditionally with ZERO capabilities required (INV-PHONE-001).
    /// If a normal call was active, it is terminated immediately in favor of emergency.
    pub fn dial_emergency(&mut self, number: &[u8]) -> Result<u32, PhoneCallError> {
        if number.len() > MAX_PHONE_NUMBER_LEN {
            return Err(PhoneCallError::NumberTooLong);
        }
        if !Self::is_emergency_number(number) {
            return Err(PhoneCallError::InvalidEmergencyNumber);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        let mut num_buf = [0u8; MAX_PHONE_NUMBER_LEN];
        num_buf[..number.len()].copy_from_slice(number);

        let session = InCallSession {
            call_id,
            number: num_buf,
            number_len: number.len(),
            is_emergency: true,
            state: CallState::EmergencyActive,
            audio_route: AudioRoute::Speakerphone, // Emergency defaults to speaker
            is_muted: false,
            is_held: false,
            dtmf_digits: [0u8; MAX_DTMF_BUFFER],
            dtmf_count: 0,
            duration_seconds: 0,
            plane: PlaneId::HardwareIsolated, // Rendered on isolated hardware plane
        };

        self.current_session = Some(session);
        Ok(call_id)
    }

    // ── Standard Cellular Voice Call (INV-PHONE-003, REQ-APP-PHONE-001) ───────

    /// Initiates an outgoing call. Requires CAP_APP_PHONE_CALLS.
    pub fn dial(
        &mut self,
        number: &[u8],
        caller_caps: u64,
    ) -> Result<u32, PhoneCallError> {
        if (caller_caps & CAP_APP_PHONE_CALLS) == 0 {
            return Err(PhoneCallError::MissingCallCapability);
        }
        if number.len() > MAX_PHONE_NUMBER_LEN {
            return Err(PhoneCallError::NumberTooLong);
        }
        if self.is_call_active() {
            return Err(PhoneCallError::CallAlreadyActive);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        let mut num_buf = [0u8; MAX_PHONE_NUMBER_LEN];
        num_buf[..number.len()].copy_from_slice(number);

        let session = InCallSession {
            call_id,
            number: num_buf,
            number_len: number.len(),
            is_emergency: false,
            state: CallState::Dialing,
            audio_route: AudioRoute::Earpiece,
            is_muted: false,
            is_held: false,
            dtmf_digits: [0u8; MAX_DTMF_BUFFER],
            dtmf_count: 0,
            duration_seconds: 0,
            plane: PlaneId::HardwareIsolated, // INV-PHONE-002
        };

        self.current_session = Some(session);
        Ok(call_id)
    }

    /// Receives an incoming call from the cellular modem HAL.
    pub fn incoming_call(
        &mut self,
        number: &[u8],
        caller_caps: u64,
    ) -> Result<u32, PhoneCallError> {
        if (caller_caps & CAP_APP_PHONE_CALLS) == 0 {
            return Err(PhoneCallError::MissingCallCapability);
        }
        if number.len() > MAX_PHONE_NUMBER_LEN {
            return Err(PhoneCallError::NumberTooLong);
        }
        if self.is_call_active() {
            return Err(PhoneCallError::CallAlreadyActive);
        }

        let call_id = self.next_call_id;
        self.next_call_id += 1;

        let mut num_buf = [0u8; MAX_PHONE_NUMBER_LEN];
        num_buf[..number.len()].copy_from_slice(number);

        let session = InCallSession {
            call_id,
            number: num_buf,
            number_len: number.len(),
            is_emergency: false,
            state: CallState::RingingIncoming,
            audio_route: AudioRoute::Earpiece,
            is_muted: false,
            is_held: false,
            dtmf_digits: [0u8; MAX_DTMF_BUFFER],
            dtmf_count: 0,
            duration_seconds: 0,
            plane: PlaneId::HardwareIsolated,
        };

        self.current_session = Some(session);
        Ok(call_id)
    }

    /// Answers incoming call.
    pub fn answer_call(&mut self) -> Result<(), PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        if session.state != CallState::RingingIncoming {
            return Err(PhoneCallError::InvalidStateTransition);
        }
        session.state = CallState::Active;
        Ok(())
    }

    /// Remote party answered outgoing call.
    pub fn remote_connected(&mut self) -> Result<(), PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        if session.state != CallState::Dialing && session.state != CallState::RingingOutgoing {
            return Err(PhoneCallError::InvalidStateTransition);
        }
        session.state = CallState::Active;
        Ok(())
    }

    // ── In-Call Controls (INV-PHONE-002, REQ-APP-PHONE-004) ───────────────────

    pub fn toggle_mute(&mut self) -> Result<bool, PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        if !session.state.is_in_call() {
            return Err(PhoneCallError::InvalidStateTransition);
        }
        session.is_muted = !session.is_muted;
        Ok(session.is_muted)
    }

    pub fn toggle_speaker(&mut self) -> Result<AudioRoute, PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        if !session.state.is_in_call() {
            return Err(PhoneCallError::InvalidStateTransition);
        }
        session.audio_route = match session.audio_route {
            AudioRoute::Speakerphone => AudioRoute::Earpiece,
            _ => AudioRoute::Speakerphone,
        };
        Ok(session.audio_route)
    }

    pub fn toggle_hold(&mut self) -> Result<bool, PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        if session.is_emergency {
            // Cannot put emergency call on hold
            return Err(PhoneCallError::InvalidStateTransition);
        }
        match session.state {
            CallState::Active => {
                session.state = CallState::Holding;
                session.is_held = true;
                Ok(true)
            }
            CallState::Holding => {
                session.state = CallState::Active;
                session.is_held = false;
                Ok(false)
            }
            _ => Err(PhoneCallError::InvalidStateTransition),
        }
    }

    pub fn send_dtmf_digit(&mut self, digit: u8) -> Result<(), PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        if !session.state.is_in_call() {
            return Err(PhoneCallError::InvalidStateTransition);
        }
        if session.dtmf_count >= MAX_DTMF_BUFFER {
            return Err(PhoneCallError::DtmfBufferFull);
        }
        session.dtmf_digits[session.dtmf_count] = digit;
        session.dtmf_count += 1;
        Ok(())
    }

    pub fn tick_duration(&mut self, seconds: u32) {
        if let Some(ref mut s) = self.current_session {
            if s.state.is_in_call() {
                s.duration_seconds = s.duration_seconds.saturating_add(seconds);
            }
        }
    }

    /// Terminates call and clears in-call session. Returns call duration in seconds.
    pub fn end_call(&mut self) -> Result<u32, PhoneCallError> {
        let session = self.current_session.as_mut().ok_or(PhoneCallError::NoActiveCall)?;
        session.state = CallState::Terminated;
        let duration = session.duration_seconds;
        self.current_session = None;
        Ok(duration)
    }

    pub fn update_proximity(&mut self, is_near: bool) {
        self.earpiece_proximity_near = is_near;
    }

    pub fn is_screen_blanked_by_proximity(&self) -> bool {
        self.is_call_active() && self.earpiece_proximity_near
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> PhoneCallEngine {
        PhoneCallEngine::new()
    }

    #[test]
    // INV-PHONE-001: Emergency dialer works with 0 capabilities and recognizes numbers
    fn test_emergency_dialer_zero_capabilities() {
        let mut engine = make_engine();
        assert!(PhoneCallEngine::is_emergency_number(b"112"));
        assert!(PhoneCallEngine::is_emergency_number(b"911"));
        assert!(PhoneCallEngine::is_emergency_number(b"100"));
        assert!(!PhoneCallEngine::is_emergency_number(b"1234567890"));

        // Dial 112 with zero capabilities
        let call_id = engine.dial_emergency(b"112").unwrap();
        assert!(call_id > 0);

        let session = engine.current_session().unwrap();
        assert!(session.is_emergency);
        assert_eq!(session.state, CallState::EmergencyActive);
        assert_eq!(session.plane, PlaneId::HardwareIsolated);
        assert_eq!(session.audio_route, AudioRoute::Speakerphone);
        assert!(engine.preempts_media_playback());

        // Invalid emergency number rejected
        let mut e2 = make_engine();
        let err = e2.dial_emergency(b"9999");
        assert_eq!(err, Err(PhoneCallError::InvalidEmergencyNumber));
    }

    #[test]
    // Emergency call preempts an ongoing regular call
    fn test_emergency_preempts_regular_call() {
        let mut engine = make_engine();
        engine.dial(b"+919876543210", CAP_APP_PHONE_CALLS).unwrap();
        assert!(!engine.current_session().unwrap().is_emergency);

        // Emergency dial immediately preempts
        engine.dial_emergency(b"100").unwrap();
        let session = engine.current_session().unwrap();
        assert!(session.is_emergency);
        assert_eq!(session.number_as_str().unwrap(), "100");
    }

    #[test]
    // INV-PHONE-003: Dial requires CAP_APP_PHONE_CALLS
    fn test_dial_requires_capability() {
        let mut engine = make_engine();
        let err = engine.dial(b"+919876543210", 0);
        assert_eq!(err, Err(PhoneCallError::MissingCallCapability));

        let ok = engine.dial(b"+919876543210", CAP_APP_PHONE_CALLS);
        assert!(ok.is_ok());
    }

    #[test]
    // Outgoing call lifecycle: Dial -> Connected -> Active -> End
    fn test_outgoing_call_lifecycle() {
        let mut engine = make_engine();
        engine.dial(b"+919876543210", CAP_APP_PHONE_CALLS).unwrap();
        assert_eq!(engine.current_session().unwrap().state, CallState::Dialing);

        engine.remote_connected().unwrap();
        assert_eq!(engine.current_session().unwrap().state, CallState::Active);

        engine.tick_duration(45);
        assert_eq!(engine.current_session().unwrap().duration_seconds, 45);

        let duration = engine.end_call().unwrap();
        assert_eq!(duration, 45);
        assert!(!engine.is_call_active());
        assert!(engine.current_session().is_none());
    }

    #[test]
    // Incoming call lifecycle: Incoming -> Answer -> Active -> End
    fn test_incoming_call_lifecycle() {
        let mut engine = make_engine();
        engine.incoming_call(b"+911122334455", CAP_APP_PHONE_CALLS).unwrap();
        assert_eq!(engine.current_session().unwrap().state, CallState::RingingIncoming);

        engine.answer_call().unwrap();
        assert_eq!(engine.current_session().unwrap().state, CallState::Active);

        engine.end_call().unwrap();
        assert!(!engine.is_call_active());
    }

    #[test]
    // INV-PHONE-002: In-call controls (mute, speaker, hold, DTMF) on HardwareIsolated plane
    fn test_in_call_controls() {
        let mut engine = make_engine();
        engine.dial(b"+919876543210", CAP_APP_PHONE_CALLS).unwrap();
        engine.remote_connected().unwrap();

        // Check plane
        assert_eq!(engine.current_session().unwrap().plane, PlaneId::HardwareIsolated);

        // Mute toggle
        assert!(!engine.current_session().unwrap().is_muted);
        assert!(engine.toggle_mute().unwrap());
        assert!(engine.current_session().unwrap().is_muted);
        assert!(!engine.toggle_mute().unwrap());

        // Speaker toggle
        assert_eq!(engine.toggle_speaker().unwrap(), AudioRoute::Speakerphone);
        assert_eq!(engine.toggle_speaker().unwrap(), AudioRoute::Earpiece);

        // Hold toggle
        assert!(engine.toggle_hold().unwrap());
        assert_eq!(engine.current_session().unwrap().state, CallState::Holding);
        assert!(!engine.toggle_hold().unwrap());
        assert_eq!(engine.current_session().unwrap().state, CallState::Active);

        // DTMF digits
        engine.send_dtmf_digit(b'1').unwrap();
        engine.send_dtmf_digit(b'2').unwrap();
        engine.send_dtmf_digit(b'#').unwrap();
        assert_eq!(engine.current_session().unwrap().dtmf_count, 3);
        assert_eq!(&engine.current_session().unwrap().dtmf_digits[..3], b"12#");
    }

    #[test]
    // Proximity blanking
    fn test_proximity_sensor_blanking() {
        let mut engine = make_engine();
        assert!(!engine.is_screen_blanked_by_proximity());

        engine.dial(b"123", CAP_APP_PHONE_CALLS).unwrap();
        engine.update_proximity(true);
        assert!(engine.is_screen_blanked_by_proximity());

        engine.update_proximity(false);
        assert!(!engine.is_screen_blanked_by_proximity());
    }
}
