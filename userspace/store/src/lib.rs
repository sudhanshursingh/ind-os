//! Ind OS Settings & Consent Store Library (`indus-store`)
//!
//! Standards:
//! - REQ-USE-206: Settings & consent store: integrity-protected, user-owned
//! - INV-STR-001: Tamper-evident integrity protection
//! - INV-STR-002: User-owned consent authority

#![no_std]

pub mod consent;
pub mod integrity;
pub mod types;

pub use consent::SettingsAndConsentStore;
pub use integrity::IntegrityEngine;
pub use types::{
    ConsentRecord, ConsentState, SettingEntry, StoreError, CAP_STORE_ADMIN, CAP_STORE_READ,
    MAX_CONSENTS, MAX_KEY_LEN, MAX_SETTINGS,
};
