//! Block Storage Device Abstraction & In-Memory Flash Emulator
//! Compliant with Bible §8.10 (SP-25), Rule N-01, Rule N-04, and Invariant INV-FS-001.

use crate::sync::Spinlock;

pub const BLOCK_SIZE: usize = 4096;
pub const TOTAL_DEVICE_BLOCKS: usize = 32; // 32 * 4KB = 128 KB in BSS

/// Common block device interface for storage drivers
pub trait BlockDevice {
    fn read_block(&self, block_idx: u64, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), &'static str>;
    fn write_block(&mut self, block_idx: u64, buf: &[u8; BLOCK_SIZE]) -> Result<(), &'static str>;
    fn block_count(&self) -> u64;
}

/// In-memory block device simulating physical NVMe / eMMC storage with fault injection
pub struct MemBlockDevice {
    storage: [[u8; BLOCK_SIZE]; TOTAL_DEVICE_BLOCKS],
}

impl MemBlockDevice {
    pub const fn new() -> Self {
        Self {
            storage: [[0u8; BLOCK_SIZE]; TOTAL_DEVICE_BLOCKS],
        }
    }

    /// Read raw 4KB block from device
    pub fn read_block(&self, block_idx: u64, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), &'static str> {
        let idx = block_idx as usize;
        if idx >= TOTAL_DEVICE_BLOCKS {
            return Err("EIO: Block index out of range");
        }
        buf.copy_from_slice(&self.storage[idx]);
        Ok(())
    }

    /// Write raw 4KB block to device
    pub fn write_block(&mut self, block_idx: u64, buf: &[u8; BLOCK_SIZE]) -> Result<(), &'static str> {
        let idx = block_idx as usize;
        if idx >= TOTAL_DEVICE_BLOCKS {
            return Err("EIO: Block index out of range");
        }
        self.storage[idx].copy_from_slice(buf);
        Ok(())
    }

    /// Fault injection helper: flips a bit at a specific block offset to simulate silent media corruption (INV-FS-001)
    pub fn corrupt_byte(&mut self, block_idx: u64, byte_offset: usize) -> Result<(), &'static str> {
        let idx = block_idx as usize;
        if idx >= TOTAL_DEVICE_BLOCKS {
            return Err("EIO: Block index out of range");
        }
        if byte_offset >= BLOCK_SIZE {
            return Err("EIO: Byte offset out of range");
        }
        self.storage[idx][byte_offset] ^= 0xFF; // Invert byte to guarantee checksum failure
        Ok(())
    }

    pub fn block_count(&self) -> u64 {
        TOTAL_DEVICE_BLOCKS as u64
    }
}

/// Global in-memory block device singleton stored in BSS (zero stack consumption)
pub static MEM_BLOCK_DEVICE: Spinlock<MemBlockDevice> = Spinlock::new(MemBlockDevice::new());

/// Dedicated crash-simulation block device singleton stored in BSS for golden-image testing
pub static CRASH_BLOCK_DEVICE: Spinlock<MemBlockDevice> = Spinlock::new(MemBlockDevice::new());
