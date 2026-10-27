//! Kernel Key Ceremony Verification Harness (SP-55, REQ-SEC-108, DEF-003)
//!
//! Architectural Invariants:
//! - INV-CEREMONY-001: 2-of-3 threshold quorum required for release authorization
//! - INV-CEREMONY-002: Immutable ceremony audit logging
//! - INV-CEREMONY-003: Compromised key revocation and quorum reconstitution

pub const KERNEL_MAX_CUSTODIANS: usize = 3;
pub const KERNEL_REQUIRED_THRESHOLD: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelCustodianStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Copy)]
pub struct KernelCustodianKey {
    pub id: u32,
    pub name: &'static str,
    pub public_key: u32,
    pub status: KernelCustodianStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelCeremonyError {
    CustodianNotFound,
    CustodianRevoked,
    DuplicateSignature,
    InsufficientQuorum,
    InvalidSignature,
}

pub struct KernelThresholdSigningEngine {
    custodians: [Option<KernelCustodianKey>; KERNEL_MAX_CUSTODIANS],
    signatures: [Option<u32>; KERNEL_MAX_CUSTODIANS],
    sig_count: usize,
}

impl KernelThresholdSigningEngine {
    pub const fn new() -> Self {
        Self {
            custodians: [None; KERNEL_MAX_CUSTODIANS],
            signatures: [None; KERNEL_MAX_CUSTODIANS],
            sig_count: 0,
        }
    }

    /// Register designated custodian
    pub fn register_custodian(
        &mut self,
        id: u32,
        name: &'static str,
        public_key: u32,
    ) -> Result<(), KernelCeremonyError> {
        for slot in self.custodians.iter_mut() {
            if slot.is_none() {
                *slot = Some(KernelCustodianKey {
                    id,
                    name,
                    public_key,
                    status: KernelCustodianStatus::Active,
                });
                return Ok(());
            }
        }
        Err(KernelCeremonyError::InsufficientQuorum)
    }

    /// Submit signature from a custodian (INV-CEREMONY-001)
    pub fn submit_signature(
        &mut self,
        custodian_id: u32,
        signature: u32,
    ) -> Result<(), KernelCeremonyError> {
        let mut found = false;
        for slot in self.custodians.iter() {
            if let Some(c) = slot {
                if c.id == custodian_id {
                    if c.status == KernelCustodianStatus::Revoked {
                        return Err(KernelCeremonyError::CustodianRevoked);
                    }
                    if signature != c.public_key ^ 0x516E {
                        return Err(KernelCeremonyError::InvalidSignature);
                    }
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(KernelCeremonyError::CustodianNotFound);
        }

        for s in self.signatures.iter() {
            if *s == Some(custodian_id) {
                return Err(KernelCeremonyError::DuplicateSignature);
            }
        }

        if self.sig_count < KERNEL_MAX_CUSTODIANS {
            self.signatures[self.sig_count] = Some(custodian_id);
            self.sig_count += 1;
            Ok(())
        } else {
            Err(KernelCeremonyError::InsufficientQuorum)
        }
    }

    /// Check if 2-of-3 threshold quorum is satisfied (INV-CEREMONY-001)
    pub fn is_quorum_satisfied(&self) -> bool {
        self.sig_count >= KERNEL_REQUIRED_THRESHOLD
    }

    /// Revoke compromised custodian key (INV-CEREMONY-003)
    pub fn revoke_custodian(&mut self, custodian_id: u32) -> Result<(), KernelCeremonyError> {
        for slot in self.custodians.iter_mut() {
            if let Some(ref mut c) = slot {
                if c.id == custodian_id {
                    c.status = KernelCustodianStatus::Revoked;
                    return Ok(());
                }
            }
        }
        Err(KernelCeremonyError::CustodianNotFound)
    }

    pub fn reset_signatures(&mut self) {
        self.signatures = [None; KERNEL_MAX_CUSTODIANS];
        self.sig_count = 0;
    }

    pub fn signature_count(&self) -> usize {
        self.sig_count
    }
}
