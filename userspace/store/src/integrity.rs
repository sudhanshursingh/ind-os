//! Ind OS Store Integrity Protection Engine
//!
//! Standards:
//! - INV-STR-001: Tamper-evident integrity protection

use crate::types::{ConsentRecord, ConsentState, SettingEntry, StoreError};

pub struct IntegrityEngine;

impl IntegrityEngine {
    // Pure Rust deterministic FNV-1a 32-bit hash
    pub fn compute_setting_checksum(key: &str, value: u32) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        for &b in key.as_bytes() {
            hash ^= b as u32;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash ^= value;
        hash = hash.wrapping_mul(0x01000193);
        hash
    }

    pub fn compute_consent_checksum(
        service_id: u32,
        permission_id: u32,
        state: ConsentState,
        timestamp_ms: u64,
    ) -> u32 {
        let mut hash: u32 = 0x811c9dc5;
        hash ^= service_id;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= permission_id;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= match state {
            ConsentState::Granted => 1,
            ConsentState::Denied => 2,
            ConsentState::PromptAlways => 3,
            ConsentState::Revoked => 4,
        };
        hash = hash.wrapping_mul(0x01000193);
        hash ^= (timestamp_ms & 0xffffffff) as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash ^= ((timestamp_ms >> 32) & 0xffffffff) as u32;
        hash = hash.wrapping_mul(0x01000193);
        hash
    }

    pub fn verify_setting(entry: &SettingEntry) -> Result<(), StoreError> {
        let expected = Self::compute_setting_checksum(entry.key, entry.value);
        if entry.checksum != expected {
            return Err(StoreError::IntegrityViolation);
        }
        Ok(())
    }

    pub fn verify_consent(record: &ConsentRecord) -> Result<(), StoreError> {
        let expected = Self::compute_consent_checksum(
            record.service_id,
            record.permission_id,
            record.state,
            record.timestamp_ms,
        );
        if record.checksum != expected {
            return Err(StoreError::IntegrityViolation);
        }
        Ok(())
    }
}
