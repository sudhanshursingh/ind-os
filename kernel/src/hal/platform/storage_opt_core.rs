//! Kernel Hardware Storage & IndFS Page Cache Optimization (SP-112, REQ-STR-101, REQ-PERF-104)
//!
//! Standards: Bible §8.16; INV-STORE-OPT-001..003
//! Rule N-01: Zero external crates

pub const KERNEL_CAP_STORAGE_OPT: u64 = 0x0000_0000_0000_0200;
pub const KERNEL_STORAGE_PAGE_CACHE_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelStorageOptError {
    PermissionDenied,
    CacheMiss,
    CacheFull,
    InvalidLba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelCachedBlock {
    pub lba: u64,
    pub dirty: bool,
    pub crc32: u32,
    pub access_count: u32,
}

impl KernelCachedBlock {
    pub const fn empty() -> Self {
        Self { lba: 0, dirty: false, crc32: 0, access_count: 0 }
    }
}

pub struct KernelStorageOptHarness {
    cache: [KernelCachedBlock; KERNEL_STORAGE_PAGE_CACHE_SIZE],
    cache_len: usize,
    hits: u64,
    misses: u64,
}

impl KernelStorageOptHarness {
    pub const fn new() -> Self {
        Self {
            cache: [KernelCachedBlock::empty(); KERNEL_STORAGE_PAGE_CACHE_SIZE],
            cache_len: 0,
            hits: 0,
            misses: 0,
        }
    }

    pub fn insert_block(&mut self, lba: u64, crc32: u32, caps: u64) -> Result<(), KernelStorageOptError> {
        if (caps & KERNEL_CAP_STORAGE_OPT) == 0 {
            return Err(KernelStorageOptError::PermissionDenied);
        }
        for i in 0..self.cache_len {
            if self.cache[i].lba == lba {
                self.cache[i].crc32 = crc32;
                self.cache[i].access_count += 1;
                return Ok(());
            }
        }
        if self.cache_len >= KERNEL_STORAGE_PAGE_CACHE_SIZE {
            let mut min_idx = 0;
            let mut min_count = u32::MAX;
            for i in 0..self.cache_len {
                if self.cache[i].access_count < min_count {
                    min_count = self.cache[i].access_count;
                    min_idx = i;
                }
            }
            self.cache[min_idx] = KernelCachedBlock { lba, dirty: false, crc32, access_count: 1 };
            return Ok(());
        }
        self.cache[self.cache_len] = KernelCachedBlock { lba, dirty: false, crc32, access_count: 1 };
        self.cache_len += 1;
        Ok(())
    }

    pub fn lookup_block(&mut self, lba: u64) -> Result<u32, KernelStorageOptError> {
        for i in 0..self.cache_len {
            if self.cache[i].lba == lba {
                self.cache[i].access_count += 1;
                self.hits += 1;
                return Ok(self.cache[i].crc32);
            }
        }
        self.misses += 1;
        Err(KernelStorageOptError::CacheMiss)
    }

    pub fn hit_rate_permille(&self) -> u32 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0;
        }
        ((self.hits * 1000) / total) as u32
    }

    pub fn cache_len(&self) -> usize {
        self.cache_len
    }
}
