//! Sprint SP-122: Beta Cohort Planning & Consent-First Telemetry (Userspace Model)
//!
//! Constitutional Invariants:
//! - PRD-001 §7.4: Consent-first local telemetry & privacy posture
//! - INV-BETA-001: Opt-In Local Telemetry & Instant Revocation
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in userspace

pub const MAX_LOCAL_EVENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryConsentStatus {
    DefaultDisabled,
    OptedInUserConsent,
    RevokedWiped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryError {
    ConsentNotGranted,
    NetworkEgressDenied,
    BufferFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTelemetryEvent {
    pub event_id: u32,
    pub timestamp_sec: u32,
    pub subsystem_tag: [u8; 8],
    pub payload_code: u32,
}

impl LocalTelemetryEvent {
    pub const fn empty() -> Self {
        Self {
            event_id: 0,
            timestamp_sec: 0,
            subsystem_tag: [0; 8],
            payload_code: 0,
        }
    }
}

pub struct BetaTelemetryEngine {
    consent: TelemetryConsentStatus,
    local_buffer: [LocalTelemetryEvent; MAX_LOCAL_EVENTS],
    event_count: usize,
    dispatched_packets: usize,
}

impl BetaTelemetryEngine {
    pub const fn new() -> Self {
        Self {
            consent: TelemetryConsentStatus::DefaultDisabled,
            local_buffer: [LocalTelemetryEvent::empty(); MAX_LOCAL_EVENTS],
            event_count: 0,
            dispatched_packets: 0,
        }
    }

    pub fn grant_user_consent(&mut self) {
        self.consent = TelemetryConsentStatus::OptedInUserConsent;
    }

    pub fn revoke_and_wipe(&mut self) {
        self.consent = TelemetryConsentStatus::RevokedWiped;
        self.event_count = 0;
        self.local_buffer = [LocalTelemetryEvent::empty(); MAX_LOCAL_EVENTS];
    }

    pub fn record_local_event(&mut self, event: LocalTelemetryEvent) -> Result<(), TelemetryError> {
        if self.event_count >= MAX_LOCAL_EVENTS {
            return Err(TelemetryError::BufferFull);
        }
        self.local_buffer[self.event_count] = event;
        self.event_count += 1;
        Ok(())
    }

    pub fn attempt_network_egress(&mut self) -> Result<usize, TelemetryError> {
        // INV-BETA-001: Strict consent check
        if self.consent != TelemetryConsentStatus::OptedInUserConsent {
            return Err(TelemetryError::NetworkEgressDenied);
        }

        let sent = self.event_count;
        self.dispatched_packets += sent;
        self.event_count = 0;
        Ok(sent)
    }

    pub fn consent_status(&self) -> TelemetryConsentStatus {
        self.consent
    }

    pub fn pending_count(&self) -> usize {
        self.event_count
    }
}
