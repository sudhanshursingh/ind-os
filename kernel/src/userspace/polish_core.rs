//! Kernel UI Polish & Adversarial Security Auditing (SP-69, REQ-SEC-109, REQ-UX-306)
//!
//! Architectural Invariants:
//! - INV-POLISH-001: Anti-tapjacking and anti-overlay protection
//! - INV-POLISH-002: Faked / spoofed consent rejection
//! - INV-POLISH-003: Universal Why-Engine explanation coverage

pub const KERNEL_EXPECTED_CONSENT_BROKER_MAGIC: u32 = 0x504F4C53; // 'POLS'

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelPolishSecurityError {
    TapjackingAttemptBlocked,
    ConsentSignatureInvalid,
    UntracedActionDenied,
    TableFull,
}

pub struct KernelAdversarialUiHarness {
    blocked_tapjacks: u32,
    rejected_faked_consents: u32,
    explanation_checks: u32,
}

impl KernelAdversarialUiHarness {
    pub const fn new() -> Self {
        Self {
            blocked_tapjacks: 0,
            rejected_faked_consents: 0,
            explanation_checks: 0,
        }
    }

    pub fn audit_input_touch(
        &mut self,
        target_layer_z: u8,
        active_overlay_z: Option<u8>,
    ) -> Result<(), KernelPolishSecurityError> {
        if let Some(overlay_z) = active_overlay_z {
            if overlay_z > target_layer_z {
                self.blocked_tapjacks += 1;
                return Err(KernelPolishSecurityError::TapjackingAttemptBlocked);
            }
        }
        Ok(())
    }

    pub fn verify_consent_token(
        &mut self,
        broker_token: u32,
        is_genuine: bool,
    ) -> Result<(), KernelPolishSecurityError> {
        if broker_token != KERNEL_EXPECTED_CONSENT_BROKER_MAGIC || !is_genuine {
            self.rejected_faked_consents += 1;
            return Err(KernelPolishSecurityError::ConsentSignatureInvalid);
        }
        Ok(())
    }

    pub fn verify_action_explanation(
        &mut self,
        has_causal_token: bool,
    ) -> Result<(), KernelPolishSecurityError> {
        if !has_causal_token {
            return Err(KernelPolishSecurityError::UntracedActionDenied);
        }
        self.explanation_checks += 1;
        Ok(())
    }

    pub fn blocked_tapjacks(&self) -> u32 {
        self.blocked_tapjacks
    }

    pub fn rejected_faked_consents(&self) -> u32 {
        self.rejected_faked_consents
    }

    pub fn explanation_checks(&self) -> u32 {
        self.explanation_checks
    }
}
