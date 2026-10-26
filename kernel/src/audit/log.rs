//! Cryptographic Append-Only Audit Log with SHA-256 Hash Chaining (Audit v1)
//! Compliant with Bible §8.9 (SP-20), §10, §14 and Invariant INV-AUD-001.

use crate::audit::entry::{AuditEntry, AuditEventType};
use crate::capability::rights::CapRights;
use crate::task::ProcessId;

/// Maximum number of audit records stored in bounded static kernel memory
pub const MAX_AUDIT_ENTRIES: usize = 128;

/// Audit log structure enforcing cryptographic hash chaining (INV-AUD-001)
pub struct AuditLog {
    entries: [AuditEntry; MAX_AUDIT_ENTRIES],
    count: usize,
}

impl AuditLog {
    /// Creates a new uninitialized audit log
    pub const fn new() -> Self {
        Self {
            entries: [AuditEntry::empty(); MAX_AUDIT_ENTRIES],
            count: 0,
        }
    }

    /// Initializes the audit log by creating the cryptographic Genesis entry
    pub fn init(&mut self) {
        if self.count == 0 {
            let mut genesis = AuditEntry {
                sequence: 0,
                timestamp: 0,
                event_type: AuditEventType::Genesis,
                subject_pid: 0,
                target_id: 0,
                rights: CapRights::NONE,
                prev_hash: [0u8; 32],
                entry_hash: [0u8; 32],
            };
            genesis.entry_hash = genesis.compute_hash();
            self.entries[0] = genesis;
            self.count = 1;
        }
    }

    /// Appends a new audit record to the hash chain
    pub fn record(
        &mut self,
        event_type: AuditEventType,
        subject_pid: ProcessId,
        target_id: u32,
        rights: CapRights,
        timestamp: u64,
    ) -> Result<u64, i64> {
        if self.count == 0 {
            self.init();
        }

        if self.count >= MAX_AUDIT_ENTRIES {
            return Err(-1); // Log capacity exceeded
        }

        let prev_hash = self.entries[self.count - 1].entry_hash;
        let seq = self.count as u64;

        let mut entry = AuditEntry {
            sequence: seq,
            timestamp,
            event_type,
            subject_pid,
            target_id,
            rights,
            prev_hash,
            entry_hash: [0u8; 32],
        };
        entry.entry_hash = entry.compute_hash();

        self.entries[self.count] = entry;
        self.count += 1;

        Ok(seq)
    }

    /// Total number of recorded audit entries
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns whether the log is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Retrieves an entry by index
    pub fn get_entry(&self, index: usize) -> Option<&AuditEntry> {
        if index < self.count {
            Some(&self.entries[index])
        } else {
            None
        }
    }

    /// Retrieves the most recent entry
    pub fn last_entry(&self) -> Option<&AuditEntry> {
        if self.count > 0 {
            Some(&self.entries[self.count - 1])
        } else {
            None
        }
    }

    /// Verifies the cryptographic integrity of the entire audit chain (INV-AUD-001)
    ///
    /// Returns Ok(()) if all entries have valid hashes and the chain is unbroken.
    /// Returns Err((corrupted_index, reason)) if tamper or corruption is detected.
    pub fn verify_integrity(&self) -> Result<(), (usize, &'static str)> {
        if self.count == 0 {
            return Ok(());
        }

        // Verify Genesis entry
        let genesis = &self.entries[0];
        if genesis.prev_hash != [0u8; 32] {
            return Err((0, "Genesis prev_hash is non-zero"));
        }
        if genesis.compute_hash() != genesis.entry_hash {
            return Err((0, "Genesis hash mismatch / corrupted"));
        }

        // Verify hash chain
        for i in 1..self.count {
            let prev = &self.entries[i - 1];
            let curr = &self.entries[i];

            if curr.prev_hash != prev.entry_hash {
                return Err((i, "Hash chain link broken: prev_hash mismatch"));
            }
            if curr.compute_hash() != curr.entry_hash {
                return Err((i, "Entry content hash mismatch: payload modified"));
            }
        }

        Ok(())
    }

    /// Artificially introduces a 1-bit tamper for testing tamper resistance (INV-AUD-001)
    pub fn tamper_flip_byte(&mut self, index: usize, byte_offset: usize) -> Result<(), &'static str> {
        if index >= self.count {
            return Err("Index out of bounds");
        }
        if byte_offset >= 32 {
            return Err("Byte offset out of bounds");
        }
        self.entries[index].entry_hash[byte_offset] ^= 0x01;
        Ok(())
    }

    /// Restores an entry after a tamper test
    pub fn restore_entry(&mut self, index: usize, entry: AuditEntry) -> Result<(), &'static str> {
        if index >= self.count {
            return Err("Index out of bounds");
        }
        self.entries[index] = entry;
        Ok(())
    }
}
