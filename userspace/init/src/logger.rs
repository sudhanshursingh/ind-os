//! Structured Boot Stage Logger for Ind OS Init (PID 1)
//!
//! Constitutional References:
//! - REQ-USE-201: Boot stages mirrored to structured logs
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code

use crate::supervisor::ServiceState;

pub const MAX_BOOT_LOGS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootStageRecord {
    pub timestamp_ticks: u64,
    pub service_name: &'static str,
    pub old_state: ServiceState,
    pub new_state: ServiceState,
    pub exit_code: Option<i32>,
    pub message: &'static str,
}

pub struct StructuredBootLogger {
    records: [Option<BootStageRecord>; MAX_BOOT_LOGS],
    count: usize,
}

impl StructuredBootLogger {
    pub const fn new() -> Self {
        Self {
            records: [None; MAX_BOOT_LOGS],
            count: 0,
        }
    }

    pub fn log_transition(
        &mut self,
        timestamp_ticks: u64,
        service_name: &'static str,
        old_state: ServiceState,
        new_state: ServiceState,
        exit_code: Option<i32>,
        message: &'static str,
    ) {
        let rec = BootStageRecord {
            timestamp_ticks,
            service_name,
            old_state,
            new_state,
            exit_code,
            message,
        };
        let idx = self.count % MAX_BOOT_LOGS;
        self.records[idx] = Some(rec);
        self.count += 1;
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn get_record(&self, idx: usize) -> Option<&BootStageRecord> {
        if idx >= self.count.min(MAX_BOOT_LOGS) {
            return None;
        }
        self.records[idx].as_ref()
    }
}
