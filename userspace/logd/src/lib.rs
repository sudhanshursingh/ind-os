//! Ind OS Unified Log Bus & Diagnostics (`indus-logd`)
//!
//! Standards:
//! - REQ-USE-204: Unified log bus, redaction filters, `why` command
//! - INV-LOG-001: Privacy redaction guarantee
//! - INV-LOG-002: Explainability & root cause reconstruction

#![no_std]

pub mod bus;
pub mod filter;
pub mod types;
pub mod why;

pub use bus::LogBus;
pub use filter::RedactionFilter;
pub use types::{
    DiagnosticCause, DiagnosticRecord, LogLevel, LogMessage, StoredLogEntry,
    MAX_DIAGNOSTIC_RECORDS, MAX_LOG_CAPACITY, MAX_LOG_MESSAGE_LEN,
};
pub use why::WhyExplanationEngine;
