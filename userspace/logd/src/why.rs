//! Ind OS `why` Diagnostic Explanation Engine
//!
//! Standards:
//! - REQ-USE-204: `why` command (precursor of the explanation engine)
//! - INV-LOG-002: Explainability & root cause reconstruction

use crate::types::{DiagnosticCause, DiagnosticRecord, MAX_DIAGNOSTIC_RECORDS};

pub struct WhyExplanationEngine {
    records: [Option<DiagnosticRecord>; MAX_DIAGNOSTIC_RECORDS],
    count: usize,
}

impl WhyExplanationEngine {
    pub const fn new() -> Self {
        Self {
            records: [None; MAX_DIAGNOSTIC_RECORDS],
            count: 0,
        }
    }

    pub fn record_failure(
        &mut self,
        event_id: u32,
        service_name: &'static str,
        cause: DiagnosticCause,
        explanation: &'static str,
        recovery_action: &'static str,
    ) {
        let rec = DiagnosticRecord {
            event_id,
            service_name,
            cause,
            explanation,
            recovery_action,
        };

        if self.count < MAX_DIAGNOSTIC_RECORDS {
            self.records[self.count] = Some(rec);
            self.count += 1;
        } else {
            // Overwrite oldest
            self.records[0] = Some(rec);
        }
    }

    pub fn explain(&self, event_id: u32) -> Option<&DiagnosticRecord> {
        for slot in self.records.iter() {
            if let Some(ref rec) = slot {
                if rec.event_id == event_id {
                    return Some(rec);
                }
            }
        }
        None
    }

    pub fn explain_service(&self, service_name: &str) -> Option<&DiagnosticRecord> {
        for slot in self.records.iter() {
            if let Some(ref rec) = slot {
                if rec.service_name == service_name {
                    return Some(rec);
                }
            }
        }
        None
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
