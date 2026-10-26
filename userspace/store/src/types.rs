//! Ind OS Settings & Consent Store Types
//!
//! Standards:
//! - REQ-USE-206: Settings & consent store
//! - INV-STR-001: Tamper-evident integrity protection
//! - INV-STR-002: User-owned consent authority

pub const MAX_KEY_LEN: usize = 32;
pub const MAX_SETTINGS: usize = 32;
pub const MAX_CONSENTS: usize = 32;

pub const CAP_STORE_READ: u32 = 0x01;
pub const CAP_STORE_ADMIN: u32 = 0x02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentState {
    Granted,
    Denied,
    PromptAlways,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    NotFound,
    PermissionDenied,
    IntegrityViolation,
    TableFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsentRecord {
    pub service_id: u32,
    pub permission_id: u32,
    pub state: ConsentState,
    pub timestamp_ms: u64,
    pub checksum: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingEntry {
    pub key: &'static str,
    pub value: u32,
    pub checksum: u32,
}
