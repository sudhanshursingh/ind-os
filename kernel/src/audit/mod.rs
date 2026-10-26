//! Cryptographic Audit Subsystem (Audit v1)
//! Compliant with Bible §8.9 (SP-20), §10, §14 and Invariant INV-AUD-001.

pub mod entry;
pub mod log;
pub mod sha256;

use crate::sync::spinlock::Spinlock;
pub use entry::{AuditEntry, AuditEventType};
pub use log::{AuditLog, MAX_AUDIT_ENTRIES};
pub use sha256::Sha256;

/// Global append-only cryptographic audit log
pub static AUDIT_LOG: Spinlock<AuditLog> = Spinlock::new(AuditLog::new());
