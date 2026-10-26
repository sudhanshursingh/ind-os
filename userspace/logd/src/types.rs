//! Ind OS Logging & Diagnostics Types
//!
//! Standards:
//! - REQ-USE-204: Unified log bus, redaction filters, `why` command
//! - INV-LOG-001: Privacy redaction guarantee
//! - INV-LOG-002: Explainability & root cause reconstruction

pub const MAX_LOG_MESSAGE_LEN: usize = 128;
pub const MAX_LOG_CAPACITY: usize = 64;
pub const MAX_DIAGNOSTIC_RECORDS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    SecurityAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogMessage {
    pub bytes: [u8; MAX_LOG_MESSAGE_LEN],
    pub len: usize,
}

impl LogMessage {
    pub const fn empty() -> Self {
        Self {
            bytes: [0u8; MAX_LOG_MESSAGE_LEN],
            len: 0,
        }
    }

    pub fn from_str(text: &str) -> Self {
        let mut msg = Self::empty();
        let b = text.as_bytes();
        let copy_len = if b.len() > MAX_LOG_MESSAGE_LEN { MAX_LOG_MESSAGE_LEN } else { b.len() };
        let mut i = 0;
        while i < copy_len {
            msg.bytes[i] = b[i];
            i += 1;
        }
        msg.len = copy_len;
        msg
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredLogEntry {
    pub timestamp_ms: u64,
    pub level: LogLevel,
    pub source_id: u16, // 1=kernel, 2=init, 3=devmgr, 4=vfs, 5=shell
    pub event_id: u32,
    pub message: LogMessage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCause {
    ProcessCrash,
    PermissionDenied,
    OutOfMemory,
    HardwareFault,
    ConfigurationError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticRecord {
    pub event_id: u32,
    pub service_name: &'static str,
    pub cause: DiagnosticCause,
    pub explanation: &'static str,
    pub recovery_action: &'static str,
}
