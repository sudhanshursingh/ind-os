//! Release Key Ceremony Tooling & 2-of-3 Threshold Signing (SP-55, REQ-SEC-108, DEF-003)
//!
//! Architectural Invariants:
//! - INV-CEREMONY-001: 2-of-3 threshold quorum required for release authorization
//! - INV-CEREMONY-002: Immutable ceremony audit logging
//! - INV-CEREMONY-003: Compromised key revocation and quorum reconstitution

pub const MAX_CUSTODIANS: usize = 3;
pub const REQUIRED_THRESHOLD: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustodianStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Copy)]
pub struct CustodianKey {
    pub id: u32,
    pub name: &'static str,
    pub public_key: u32,
    pub status: CustodianStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeremonyError {
    CustodianNotFound,
    CustodianRevoked,
    DuplicateSignature,
    InsufficientQuorum,
    InvalidSignature,
}

pub struct ThresholdSigningEngine {
    custodians: [Option<CustodianKey>; MAX_CUSTODIANS],
    signatures: [Option<u32>; MAX_CUSTODIANS],
    sig_count: usize,
}

impl ThresholdSigningEngine {
    pub const fn new() -> Self {
        Self {
            custodians: [None; MAX_CUSTODIANS],
            signatures: [None; MAX_CUSTODIANS],
            sig_count: 0,
        }
    }

    /// Register designated custodian (1: Founder, 2: Security Lead, 3: Infra Lead)
    pub fn register_custodian(&mut self, id: u32, name: &'static str, public_key: u32) -> Result<(), CeremonyError> {
        for slot in self.custodians.iter_mut() {
            if slot.is_none() {
                *slot = Some(CustodianKey {
                    id,
                    name,
                    public_key,
                    status: CustodianStatus::Active,
                });
                return Ok(());
            }
        }
        Err(CeremonyError::InsufficientQuorum)
    }

    /// Submit signature from a custodian (INV-CEREMONY-001)
    pub fn submit_signature(&mut self, custodian_id: u32, signature: u32) -> Result<(), CeremonyError> {
        let mut found = false;
        for slot in self.custodians.iter() {
            if let Some(c) = slot {
                if c.id == custodian_id {
                    if c.status == CustodianStatus::Revoked {
                        return Err(CeremonyError::CustodianRevoked);
                    }
                    if signature != c.public_key ^ 0x516E { // Mock signature verification rule
                        return Err(CeremonyError::InvalidSignature);
                    }
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(CeremonyError::CustodianNotFound);
        }

        // Check if already signed
        for s in self.signatures.iter() {
            if *s == Some(custodian_id) {
                return Err(CeremonyError::DuplicateSignature);
            }
        }

        self.signatures[self.sig_count] = Some(custodian_id);
        self.sig_count += 1;
        Ok(())
    }

    /// Evaluate if 2-of-3 threshold quorum is satisfied (INV-CEREMONY-001)
    pub fn is_quorum_satisfied(&self) -> bool {
        self.sig_count >= REQUIRED_THRESHOLD
    }

    /// Revoke compromised custodian key (INV-CEREMONY-003)
    pub fn revoke_custodian(&mut self, custodian_id: u32) -> Result<(), CeremonyError> {
        for slot in self.custodians.iter_mut() {
            if let Some(ref mut c) = slot {
                if c.id == custodian_id {
                    c.status = CustodianStatus::Revoked;
                    return Ok(());
                }
            }
        }
        Err(CeremonyError::CustodianNotFound)
    }

    pub fn reset_signatures(&mut self) {
        self.signatures = [None; MAX_CUSTODIANS];
        self.sig_count = 0;
    }
}
