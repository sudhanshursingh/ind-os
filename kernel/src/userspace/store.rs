//! Kernel Settings & Consent Store Verification Harness (SP-37)
//!
//! Architectural Invariants:
//! - REQ-USE-206: Settings & consent store: integrity-protected, user-owned
//! - INV-STR-001: Tamper-evident integrity protection
//! - INV-STR-002: User-owned consent authority

pub const KERNEL_MAX_SETTINGS: usize = 16;
pub const KERNEL_MAX_CONSENTS: usize = 16;

pub const CAP_STORE_READ: u32 = 0x01;
pub const CAP_STORE_ADMIN: u32 = 0x02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelConsentState {
    Granted,
    Denied,
    PromptAlways,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelStoreError {
    NotFound,
    PermissionDenied,
    IntegrityViolation,
    TableFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSettingEntry {
    pub key: &'static str,
    pub value: u32,
    pub checksum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelConsentRecord {
    pub service_id: u32,
    pub permission_id: u32,
    pub state: KernelConsentState,
    pub timestamp_ms: u64,
    pub checksum: u32,
}

pub struct KernelIntegrity;

impl KernelIntegrity {
    pub fn checksum_setting(key: &str, value: u32) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for &b in key.as_bytes() {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash ^= value;
        hash = hash.wrapping_mul(0x01000193);
        hash
    }

    pub fn checksum_consent(service_id: u32, perm: u32, state: KernelConsentState, ts: u64) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        hash ^= service_id;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= perm;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= match state {
            KernelConsentState::Granted => 1,
            KernelConsentState::Denied => 2,
            KernelConsentState::PromptAlways => 3,
            KernelConsentState::Revoked => 4,
        };
        hash = hash.wrapping_mul(0x01000193);
        hash ^= (ts & 0xffffffff) as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= ((ts >> 32) & 0xffffffff) as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash
    }
}

pub struct KernelSettingsAndConsentStore {
    settings: [Option<KernelSettingEntry>; KERNEL_MAX_SETTINGS],
    consents: [Option<KernelConsentRecord>; KERNEL_MAX_CONSENTS],
    setting_count: usize,
    consent_count: usize,
}

impl KernelSettingsAndConsentStore {
    pub const fn new() -> Self {
        Self {
            settings: [None; KERNEL_MAX_SETTINGS],
            consents: [None; KERNEL_MAX_CONSENTS],
            setting_count: 0,
            consent_count: 0,
        }
    }

    pub fn set_setting(&mut self, key: &'static str, value: u32, cap: u32) -> Result<(), KernelStoreError> {
        if (cap & CAP_STORE_ADMIN) == 0 {
            return Err(KernelStoreError::PermissionDenied);
        }
        let checksum = KernelIntegrity::checksum_setting(key, value);
        let entry = KernelSettingEntry { key, value, checksum };

        for slot in self.settings.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.key == key {
                    *s = entry;
                    return Ok(());
                }
            }
        }
        for slot in self.settings.iter_mut() {
            if slot.is_none() {
                *slot = Some(entry);
                self.setting_count += 1;
                return Ok(());
            }
        }
        Err(KernelStoreError::TableFull)
    }

    pub fn get_setting(&self, key: &str, cap: u32) -> Result<u32, KernelStoreError> {
        if (cap & (CAP_STORE_READ | CAP_STORE_ADMIN)) == 0 {
            return Err(KernelStoreError::PermissionDenied);
        }
        for slot in self.settings.iter() {
            if let Some(ref s) = slot {
                if s.key == key {
                    let expected = KernelIntegrity::checksum_setting(s.key, s.value);
                    if s.checksum != expected {
                        return Err(KernelStoreError::IntegrityViolation);
                    }
                    return Ok(s.value);
                }
            }
        }
        Err(KernelStoreError::NotFound)
    }

    pub fn set_consent(
        &mut self,
        service_id: u32,
        permission_id: u32,
        state: KernelConsentState,
        timestamp_ms: u64,
        cap: u32,
    ) -> Result<(), KernelStoreError> {
        if (cap & CAP_STORE_ADMIN) == 0 {
            return Err(KernelStoreError::PermissionDenied);
        }
        let checksum = KernelIntegrity::checksum_consent(service_id, permission_id, state, timestamp_ms);
        let record = KernelConsentRecord {
            service_id,
            permission_id,
            state,
            timestamp_ms,
            checksum,
        };

        for slot in self.consents.iter_mut() {
            if let Some(ref mut c) = slot {
                if c.service_id == service_id && c.permission_id == permission_id {
                    *c = record;
                    return Ok(());
                }
            }
        }
        for slot in self.consents.iter_mut() {
            if slot.is_none() {
                *slot = Some(record);
                self.consent_count += 1;
                return Ok(());
            }
        }
        Err(KernelStoreError::TableFull)
    }

    pub fn check_consent(&self, service_id: u32, permission_id: u32, cap: u32) -> Result<KernelConsentState, KernelStoreError> {
        if (cap & (CAP_STORE_READ | CAP_STORE_ADMIN)) == 0 {
            return Err(KernelStoreError::PermissionDenied);
        }
        for slot in self.consents.iter() {
            if let Some(ref c) = slot {
                if c.service_id == service_id && c.permission_id == permission_id {
                    let expected = KernelIntegrity::checksum_consent(c.service_id, c.permission_id, c.state, c.timestamp_ms);
                    if c.checksum != expected {
                        return Err(KernelStoreError::IntegrityViolation);
                    }
                    return Ok(c.state);
                }
            }
        }
        Err(KernelStoreError::NotFound)
    }

    pub fn simulate_tamper(&mut self, key: &str, corrupt_val: u32) {
        for slot in self.settings.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.key == key {
                    s.value = corrupt_val;
                    return;
                }
            }
        }
    }
}
