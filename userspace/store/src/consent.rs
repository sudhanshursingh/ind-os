//! Ind OS Settings & Consent Policy Store
//!
//! Standards:
//! - REQ-USE-206: Settings & consent store: integrity-protected, user-owned
//! - INV-STR-001: Tamper-evident integrity protection
//! - INV-STR-002: User-owned consent authority

use crate::integrity::IntegrityEngine;
use crate::types::{
    ConsentRecord, ConsentState, SettingEntry, StoreError, CAP_STORE_ADMIN, CAP_STORE_READ,
    MAX_CONSENTS, MAX_SETTINGS,
};

pub struct SettingsAndConsentStore {
    settings: [Option<SettingEntry>; MAX_SETTINGS],
    consents: [Option<ConsentRecord>; MAX_CONSENTS],
    setting_count: usize,
    consent_count: usize,
}

impl SettingsAndConsentStore {
    pub const fn new() -> Self {
        Self {
            settings: [None; MAX_SETTINGS],
            consents: [None; MAX_CONSENTS],
            setting_count: 0,
            consent_count: 0,
        }
    }

    pub fn set_setting(
        &mut self,
        key: &'static str,
        value: u32,
        cap_rights: u32,
    ) -> Result<(), StoreError> {
        // Enforce INV-STR-002: User-owned consent authority (requires admin cap)
        if (cap_rights & CAP_STORE_ADMIN) == 0 {
            return Err(StoreError::PermissionDenied);
        }

        let checksum = IntegrityEngine::compute_setting_checksum(key, value);
        let entry = SettingEntry { key, value, checksum };

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

        Err(StoreError::TableFull)
    }

    pub fn get_setting(&self, key: &str, cap_rights: u32) -> Result<u32, StoreError> {
        if (cap_rights & (CAP_STORE_READ | CAP_STORE_ADMIN)) == 0 {
            return Err(StoreError::PermissionDenied);
        }

        for slot in self.settings.iter() {
            if let Some(ref s) = slot {
                if s.key == key {
                    // Enforce INV-STR-001: Integrity verification
                    IntegrityEngine::verify_setting(s)?;
                    return Ok(s.value);
                }
            }
        }

        Err(StoreError::NotFound)
    }

    pub fn set_consent(
        &mut self,
        service_id: u32,
        permission_id: u32,
        state: ConsentState,
        timestamp_ms: u64,
        cap_rights: u32,
    ) -> Result<(), StoreError> {
        // Enforce INV-STR-002: User-owned consent authority
        if (cap_rights & CAP_STORE_ADMIN) == 0 {
            return Err(StoreError::PermissionDenied);
        }

        let checksum = IntegrityEngine::compute_consent_checksum(
            service_id,
            permission_id,
            state,
            timestamp_ms,
        );
        let record = ConsentRecord {
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

        Err(StoreError::TableFull)
    }

    pub fn check_consent(
        &self,
        service_id: u32,
        permission_id: u32,
        cap_rights: u32,
    ) -> Result<ConsentState, StoreError> {
        if (cap_rights & (CAP_STORE_READ | CAP_STORE_ADMIN)) == 0 {
            return Err(StoreError::PermissionDenied);
        }

        for slot in self.consents.iter() {
            if let Some(ref c) = slot {
                if c.service_id == service_id && c.permission_id == permission_id {
                    // Enforce INV-STR-001: Integrity verification
                    IntegrityEngine::verify_consent(c)?;
                    return Ok(c.state);
                }
            }
        }

        Err(StoreError::NotFound)
    }

    pub fn simulate_tamper_setting(&mut self, key: &str, corrupt_value: u32) {
        for slot in self.settings.iter_mut() {
            if let Some(ref mut s) = slot {
                if s.key == key {
                    s.value = corrupt_value;
                    // Do not update checksum!
                    return;
                }
            }
        }
    }
}
