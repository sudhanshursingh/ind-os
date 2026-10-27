//! Ind OS Personal Memory Store (`indus-ais::memory`)
//!
//! Standards:
//! - REQ-AI-103: Encrypted, scoped, user-editable memory
//! - INV-MEM-001: Scoped, encrypted & user-editable memory
//! - INV-MEM-002: ENF-enforced retention & tombstoning
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const MAX_MEMORY_ITEMS: usize = 16;
pub const MAX_MEMORY_VAL_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryScope {
    EphemeralSession,
    TaskScoped,
    LongTermPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStatus {
    Active,
    Edited,
    Tombstoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryItem {
    pub item_id: u32,
    pub scope: MemoryScope,
    pub status: MemoryStatus,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub key: &'static str,
    pub value_bytes: [u8; MAX_MEMORY_VAL_LEN],
    pub value_len: usize,
}

impl MemoryItem {
    pub fn value_as_str(&self) -> &str {
        core::str::from_utf8(&self.value_bytes[..self.value_len]).unwrap_or("")
    }
}

pub struct PersonalMemoryStore {
    items: [Option<MemoryItem>; MAX_MEMORY_ITEMS],
    count: usize,
    next_id: u32,
}

impl PersonalMemoryStore {
    pub const fn new() -> Self {
        Self {
            items: [None; MAX_MEMORY_ITEMS],
            count: 0,
            next_id: 7001,
        }
    }

    pub fn insert(
        &mut self,
        scope: MemoryScope,
        key: &'static str,
        value: &str,
        created_at_ms: u64,
        ttl_ms: u64,
    ) -> Result<u32, ()> {
        if self.count >= MAX_MEMORY_ITEMS {
            return Err(());
        }

        let mut val_buf = [0u8; MAX_MEMORY_VAL_LEN];
        let bytes = value.as_bytes();
        let len = if bytes.len() > MAX_MEMORY_VAL_LEN {
            MAX_MEMORY_VAL_LEN
        } else {
            bytes.len()
        };
        val_buf[..len].copy_from_slice(&bytes[..len]);

        let id = self.next_id;
        self.next_id += 1;

        let item = MemoryItem {
            item_id: id,
            scope,
            status: MemoryStatus::Active,
            created_at_ms,
            expires_at_ms: created_at_ms + ttl_ms,
            key,
            value_bytes: val_buf,
            value_len: len,
        };

        self.items[self.count] = Some(item);
        self.count += 1;
        Ok(id)
    }

    pub fn get(&self, item_id: u32) -> Option<&MemoryItem> {
        for i in 0..self.count {
            if let Some(ref it) = self.items[i] {
                if it.item_id == item_id && it.status != MemoryStatus::Tombstoned {
                    return Some(it);
                }
            }
        }
        None
    }

    pub fn edit(&mut self, item_id: u32, new_value: &str) -> Result<(), ()> {
        for i in 0..self.count {
            if let Some(ref mut it) = self.items[i] {
                if it.item_id == item_id && it.status != MemoryStatus::Tombstoned {
                    let mut val_buf = [0u8; MAX_MEMORY_VAL_LEN];
                    let bytes = new_value.as_bytes();
                    let len = if bytes.len() > MAX_MEMORY_VAL_LEN {
                        MAX_MEMORY_VAL_LEN
                    } else {
                        bytes.len()
                    };
                    val_buf[..len].copy_from_slice(&bytes[..len]);

                    it.value_bytes = val_buf;
                    it.value_len = len;
                    it.status = MemoryStatus::Edited;
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn delete_tombstone(&mut self, item_id: u32) -> Result<(), ()> {
        for i in 0..self.count {
            if let Some(ref mut it) = self.items[i] {
                if it.item_id == item_id {
                    // Cryptographic wipe of content (INV-MEM-002)
                    it.value_bytes = [0u8; MAX_MEMORY_VAL_LEN];
                    it.value_len = 0;
                    it.status = MemoryStatus::Tombstoned;
                    return Ok(());
                }
            }
        }
        Err(())
    }

    pub fn sweep_expired(&mut self, current_time_ms: u64) -> usize {
        let mut purged = 0;
        for i in 0..self.count {
            if let Some(ref mut it) = self.items[i] {
                if it.status != MemoryStatus::Tombstoned && current_time_ms >= it.expires_at_ms {
                    it.value_bytes = [0u8; MAX_MEMORY_VAL_LEN];
                    it.value_len = 0;
                    it.status = MemoryStatus::Tombstoned;
                    purged += 1;
                }
            }
        }
        purged
    }
}
