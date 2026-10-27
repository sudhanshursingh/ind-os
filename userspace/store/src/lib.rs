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
pub mod ufs_mount;

pub use consent::SettingsAndConsentStore;
pub use integrity::IntegrityEngine;
pub use ufs_mount::{
    UfsMountError, UfsMountHarness, UfsMountState,
    CAP_STORAGE_MOUNT, INDFS_BLOCK_SIZE_BYTES, INDFS_SUPERBLOCK_MAGIC,
};
pub use types::{
    ConsentRecord, ConsentState, SettingEntry, StoreError, CAP_STORE_ADMIN, CAP_STORE_READ,
    MAX_CONSENTS, MAX_KEY_LEN, MAX_SETTINGS,
};
