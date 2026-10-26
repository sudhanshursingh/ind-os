//! Ind OS Unified Log Bus Ring Buffer
//!
//! Standards:
//! - REQ-USE-204: Unified log bus
//! - INV-LOG-001: Redaction filters applied at ingestion

use crate::filter::RedactionFilter;
use crate::types::{LogLevel, StoredLogEntry, MAX_LOG_CAPACITY};

pub struct LogBus {
    entries: [Option<StoredLogEntry>; MAX_LOG_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
    total_ingested: u64,
}

impl LogBus {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_LOG_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
            total_ingested: 0,
        }
    }

    pub fn ingest(
        &mut self,
        timestamp_ms: u64,
        level: LogLevel,
        source_id: u16,
        event_id: u32,
        raw_msg: &str,
    ) {
        // Redaction filter enforced at ingestion per INV-LOG-001
        let redacted = RedactionFilter::redact(raw_msg);

        let entry = StoredLogEntry {
            timestamp_ms,
            level,
            source_id,
            event_id,
            message: redacted,
        };

        self.entries[self.tail] = Some(entry);
        self.tail = (self.tail + 1) % MAX_LOG_CAPACITY;
        if self.count < MAX_LOG_CAPACITY {
            self.count += 1;
        } else {
            // Overwritten oldest entry in ring buffer
            self.head = (self.head + 1) % MAX_LOG_CAPACITY;
        }
        self.total_ingested += 1;
    }

    pub fn latest_entry(&self) -> Option<&StoredLogEntry> {
        if self.count == 0 {
            return None;
        }
        let prev_idx = if self.tail == 0 { MAX_LOG_CAPACITY - 1 } else { self.tail - 1 };
        self.entries[prev_idx].as_ref()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn total_ingested(&self) -> u64 {
        self.total_ingested
    }
}
