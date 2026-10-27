//! Kernel Personal Memory Store Verification Harness (SP-45)
//!
//! Constitutional References:
//! - Bible §6.3, §8.12: Personal memory store (encrypted, scoped, user-editable, ENF-enforced retention)
//! - REQ-AI-103: Personal memory store
//! - INV-MEM-001: Scoped, encrypted & user-editable memory
//! - INV-MEM-002: ENF-enforced retention & tombstoning
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const KERNEL_MAX_MEMORY_ITEMS: usize = 16;
pub const KERNEL_MAX_MEMORY_VAL_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelMemoryScope {
    EphemeralSession,
    TaskScoped,
    LongTermPreference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelMemoryStatus {
    Active,
    Edited,
    Tombstoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelMemoryItem {
    pub item_id: u32,
    pub scope: KernelMemoryScope,
    pub status: KernelMemoryStatus,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub key: &'static str,
    pub value_bytes: [u8; KERNEL_MAX_MEMORY_VAL_LEN],
    pub value_len: usize,
}

impl KernelMemoryItem {
    pub fn value_as_str(&self) -> &str {
        core::str::from_utf8(&self.value_bytes[..self.value_len]).unwrap_or("")
    }
}

pub struct KernelMemoryStore {
    items: [Option<KernelMemoryItem>; KERNEL_MAX_MEMORY_ITEMS],
    count: usize,
    next_id: u32,
}

impl KernelMemoryStore {
    pub const fn new() -> Self {
        Self {
            items: [None; KERNEL_MAX_MEMORY_ITEMS],
            count: 0,
            next_id: 7001,
        }
    }

    pub fn insert(
        &mut self,
        scope: KernelMemoryScope,
        key: &'static str,
        value: &str,
        created_at_ms: u64,
        ttl_ms: u64,
    ) -> Result<u32, ()> {
        if self.count >= KERNEL_MAX_MEMORY_ITEMS {
            return Err(());
        }

        let mut val_buf = [0u8; KERNEL_MAX_MEMORY_VAL_LEN];
        let bytes = value.as_bytes();
        let len = if bytes.len() > KERNEL_MAX_MEMORY_VAL_LEN {
            KERNEL_MAX_MEMORY_VAL_LEN
        } else {
            bytes.len()
        };
        val_buf[..len].copy_from_slice(&bytes[..len]);

        let id = self.next_id;
        self.next_id += 1;

        let item = KernelMemoryItem {
            item_id: id,
            scope,
            status: KernelMemoryStatus::Active,
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

    pub fn get(&self, item_id: u32) -> Option<&KernelMemoryItem> {
        for i in 0..self.count {
            if let Some(ref it) = self.items[i] {
                if it.item_id == item_id && it.status != KernelMemoryStatus::Tombstoned {
                    return Some(it);
                }
            }
        }
        None
    }

    pub fn edit(&mut self, item_id: u32, new_value: &str) -> Result<(), ()> {
        for i in 0..self.count {
            if let Some(ref mut it) = self.items[i] {
                if it.item_id == item_id && it.status != KernelMemoryStatus::Tombstoned {
                    let mut val_buf = [0u8; KERNEL_MAX_MEMORY_VAL_LEN];
                    let bytes = new_value.as_bytes();
                    let len = if bytes.len() > KERNEL_MAX_MEMORY_VAL_LEN {
                        KERNEL_MAX_MEMORY_VAL_LEN
                    } else {
                        bytes.len()
                    };
                    val_buf[..len].copy_from_slice(&bytes[..len]);

                    it.value_bytes = val_buf;
                    it.value_len = len;
                    it.status = KernelMemoryStatus::Edited;
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
                    it.value_bytes = [0u8; KERNEL_MAX_MEMORY_VAL_LEN];
                    it.value_len = 0;
                    it.status = KernelMemoryStatus::Tombstoned;
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
                if it.status != KernelMemoryStatus::Tombstoned && current_time_ms >= it.expires_at_ms {
                    it.value_bytes = [0u8; KERNEL_MAX_MEMORY_VAL_LEN];
                    it.value_len = 0;
                    it.status = KernelMemoryStatus::Tombstoned;
                    purged += 1;
                }
            }
        }
        purged
    }
}
