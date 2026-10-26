//! Kernel Logging & Diagnostics Verification Harness (SP-36)
//!
//! Architectural Invariants:
//! - REQ-USE-204: Unified log bus, redaction filters, `why` command
//! - INV-LOG-001: Privacy redaction guarantee
//! - INV-LOG-002: Explainability & root cause reconstruction

pub const KERNEL_MAX_LOG_LEN: usize = 128;
pub const KERNEL_LOG_CAPACITY: usize = 32;
pub const KERNEL_MAX_DIAG_RECORDS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelLogLevel {
    Debug,
    Info,
    Warn,
    Error,
    SecurityAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelLogMessage {
    pub bytes: [u8; KERNEL_MAX_LOG_LEN],
    pub len: usize,
}

impl KernelLogMessage {
    pub const fn empty() -> Self {
        Self {
            bytes: [0u8; KERNEL_MAX_LOG_LEN],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

pub struct KernelRedactionFilter;

impl KernelRedactionFilter {
    pub fn redact(input: &str) -> KernelLogMessage {
        let mut out = KernelLogMessage::empty();
        let bytes = input.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len && out.len < KERNEL_MAX_LOG_LEN {
            if Self::match_prefix(bytes, i, b"token:0x") {
                Self::append(&mut out, "token:[REDACTED]");
                i += 8;
                while i < len && Self::is_hex(bytes[i]) {
                    i += 1;
                }
            } else if Self::match_prefix(bytes, i, b"key:0x") {
                Self::append(&mut out, "key:[REDACTED]");
                i += 6;
                while i < len && Self::is_hex(bytes[i]) {
                    i += 1;
                }
            } else if Self::match_prefix(bytes, i, b"secret:") {
                Self::append(&mut out, "secret:[REDACTED]");
                i += 7;
                while i < len && bytes[i] != b' ' && bytes[i] != b'\n' {
                    i += 1;
                }
            } else {
                if out.len < KERNEL_MAX_LOG_LEN {
                    out.bytes[out.len] = bytes[i];
                    out.len += 1;
                }
                i += 1;
            }
        }
        out
    }

    fn match_prefix(bytes: &[u8], idx: usize, prefix: &[u8]) -> bool {
        if idx + prefix.len() > bytes.len() {
            return false;
        }
        let mut j = 0;
        while j < prefix.len() {
            if bytes[idx + j] != prefix[j] {
                return false;
            }
            j += 1;
        }
        true
    }

    fn is_hex(b: u8) -> bool {
        (b >= b'0' && b <= b'9') || (b >= b'a' && b <= b'f') || (b >= b'A' && b <= b'F')
    }

    fn append(out: &mut KernelLogMessage, text: &str) {
        let b = text.as_bytes();
        let mut j = 0;
        while j < b.len() && out.len < KERNEL_MAX_LOG_LEN {
            out.bytes[out.len] = b[j];
            out.len += 1;
            j += 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelStoredLogEntry {
    pub timestamp_ms: u64,
    pub level: KernelLogLevel,
    pub source_id: u16,
    pub event_id: u32,
    pub message: KernelLogMessage,
}

pub struct KernelLogBus {
    entries: [Option<KernelStoredLogEntry>; KERNEL_LOG_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
}

impl KernelLogBus {
    pub const fn new() -> Self {
        Self {
            entries: [None; KERNEL_LOG_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn ingest(&mut self, timestamp_ms: u64, level: KernelLogLevel, source_id: u16, event_id: u32, msg: &str) {
        let redacted = KernelRedactionFilter::redact(msg);
        let entry = KernelStoredLogEntry {
            timestamp_ms,
            level,
            source_id,
            event_id,
            message: redacted,
        };
        self.entries[self.tail] = Some(entry);
        self.tail = (self.tail + 1) % KERNEL_LOG_CAPACITY;
        if self.count < KERNEL_LOG_CAPACITY {
            self.count += 1;
        } else {
            self.head = (self.head + 1) % KERNEL_LOG_CAPACITY;
        }
    }

    pub fn latest_entry(&self) -> Option<&KernelStoredLogEntry> {
        if self.count == 0 {
            return None;
        }
        let prev = if self.tail == 0 { KERNEL_LOG_CAPACITY - 1 } else { self.tail - 1 };
        self.entries[prev].as_ref()
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelDiagnosticCause {
    ProcessCrash,
    PermissionDenied,
    OutOfMemory,
    HardwareFault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelDiagnosticRecord {
    pub event_id: u32,
    pub service_name: &'static str,
    pub cause: KernelDiagnosticCause,
    pub explanation: &'static str,
    pub remedy: &'static str,
}

pub struct KernelWhyEngine {
    records: [Option<KernelDiagnosticRecord>; KERNEL_MAX_DIAG_RECORDS],
    count: usize,
}

impl KernelWhyEngine {
    pub const fn new() -> Self {
        Self {
            records: [None; KERNEL_MAX_DIAG_RECORDS],
            count: 0,
        }
    }

    pub fn record(
        &mut self,
        event_id: u32,
        service_name: &'static str,
        cause: KernelDiagnosticCause,
        explanation: &'static str,
        remedy: &'static str,
    ) {
        if self.count < KERNEL_MAX_DIAG_RECORDS {
            self.records[self.count] = Some(KernelDiagnosticRecord {
                event_id,
                service_name,
                cause,
                explanation,
                remedy,
            });
            self.count += 1;
        }
    }

    pub fn explain(&self, event_id: u32) -> Option<&KernelDiagnosticRecord> {
        for slot in self.records.iter() {
            if let Some(ref r) = slot {
                if r.event_id == event_id {
                    return Some(r);
                }
            }
        }
        None
    }
}
