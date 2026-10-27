//! UFS Power-Loss Resilience & Journaling Subsystem (SP-75, REQ-HW-105, REQ-STR-102)
//!
//! Architectural Invariants:
//! - INV-UFS-JRN-001: Hardware write barrier & sync cache enforcement
//! - INV-UFS-JRN-002: Atomic journal rollback & checksum verification
//! - INV-UFS-JRN-003: Recovery-first partition write protection

pub const SCSI_CMD_SYNCHRONIZE_CACHE: u8 = 0x35;
pub const CAP_STORAGE_RECOVERY: u32 = 0x0002_0000;
pub const MAX_JOURNAL_ENTRIES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UfsJournalError {
    InvalidEntry,
    ChecksumMismatch,
    PowerLossDetected,
    WriteProtectedPartition,
    PermissionDenied,
    BufferFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UfsJournalEntry {
    pub seq: u64,
    pub lba: u64,
    pub sector_count: u32,
    pub checksum: u32,
    pub committed: bool,
}

impl UfsJournalEntry {
    pub const fn empty() -> Self {
        Self {
            seq: 0,
            lba: 0,
            sector_count: 0,
            checksum: 0,
            committed: false,
        }
    }
}

pub struct UfsJournalHarness {
    entries: [UfsJournalEntry; MAX_JOURNAL_ENTRIES],
    entry_count: usize,
    sync_cache_count: u32,
    replayed_count: u32,
}

impl UfsJournalHarness {
    pub const fn new() -> Self {
        Self {
            entries: [UfsJournalEntry::empty(); MAX_JOURNAL_ENTRIES],
            entry_count: 0,
            sync_cache_count: 0,
            replayed_count: 0,
        }
    }

    pub fn append_entry(&mut self, lba: u64, sectors: u32, checksum: u32) -> Result<u64, UfsJournalError> {
        if self.entry_count >= MAX_JOURNAL_ENTRIES {
            return Err(UfsJournalError::BufferFull);
        }
        if sectors == 0 {
            return Err(UfsJournalError::InvalidEntry);
        }
        let seq = (self.entry_count as u64) + 1;
        self.entries[self.entry_count] = UfsJournalEntry {
            seq,
            lba,
            sector_count: sectors,
            checksum,
            committed: false,
        };
        self.entry_count += 1;
        Ok(seq)
    }

    pub fn commit_barrier(&mut self) -> Result<(), UfsJournalError> {
        if self.entry_count == 0 {
            return Ok(());
        }
        let last_idx = self.entry_count - 1;
        self.entries[last_idx].committed = true;
        self.sync_cache_count += 1;
        Ok(())
    }

    pub fn simulate_recovery_playback(&mut self) -> Result<u32, UfsJournalError> {
        let mut replayed = 0;
        for i in 0..self.entry_count {
            if self.entries[i].committed {
                if self.entries[i].checksum == 0 {
                    return Err(UfsJournalError::ChecksumMismatch);
                }
                replayed += 1;
            }
        }
        self.replayed_count = replayed;
        Ok(replayed)
    }

    pub fn verify_partition_write_guard(&self, lba: u64, cap_mask: u32) -> Result<(), UfsJournalError> {
        // Critical recovery/boot partitions reside at LBA < 0x2000
        if lba < 0x2000 && (cap_mask & CAP_STORAGE_RECOVERY) == 0 {
            return Err(UfsJournalError::WriteProtectedPartition);
        }
        Ok(())
    }

    pub fn sync_cache_count(&self) -> u32 {
        self.sync_cache_count
    }

    pub fn replayed_count(&self) -> u32 {
        self.replayed_count
    }
}
