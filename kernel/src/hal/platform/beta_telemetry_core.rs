//! Kernel Core for Beta Cohort Planning & Consent-First Telemetry (SP-122)
//!
//! Constitutional Invariants:
//! - PRD-001 §7.4: Consent-first local telemetry & privacy posture
//! - INV-BETA-001: Opt-In Local Telemetry & Instant Revocation
//! - Rule N-01: Zero external crates

pub const KERNEL_CAP_TELEMETRY_ADMIN: u32 = 0x8000_00B6;
pub const KERNEL_CAP_NETWORK_EGRESS: u32 = 0x0000_0040;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelTelemetryError {
    PermissionDenied,
    ConsentNotGranted,
    NetworkEgressDenied,
}

pub struct KernelBetaTelemetryHarness {
    consent_granted: bool,
    egress_attempts: usize,
    blocked_egress: usize,
    successful_egress: usize,
}

impl KernelBetaTelemetryHarness {
    pub const fn new() -> Self {
        Self {
            consent_granted: false,
            egress_attempts: 0,
            blocked_egress: 0,
            successful_egress: 0,
        }
    }

    pub fn set_user_consent(&mut self, consent: bool, caller_cap: u32) -> Result<(), KernelTelemetryError> {
        if (caller_cap & KERNEL_CAP_TELEMETRY_ADMIN) != KERNEL_CAP_TELEMETRY_ADMIN {
            return Err(KernelTelemetryError::PermissionDenied);
        }
        self.consent_granted = consent;
        Ok(())
    }

    /// Verifies that egress is strictly denied when consent is false
    pub fn dispatch_telemetry_packet(&mut self, caller_cap: u32) -> Result<(), KernelTelemetryError> {
        self.egress_attempts += 1;
        if !self.consent_granted || (caller_cap & KERNEL_CAP_NETWORK_EGRESS) != KERNEL_CAP_NETWORK_EGRESS {
            self.blocked_egress += 1;
            return Err(KernelTelemetryError::NetworkEgressDenied);
        }

        self.successful_egress += 1;
        Ok(())
    }

    pub fn is_consent_granted(&self) -> bool {
        self.consent_granted
    }

    pub fn blocked_count(&self) -> usize {
        self.blocked_egress
    }
}
