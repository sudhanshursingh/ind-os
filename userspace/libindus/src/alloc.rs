//! Bounded User Memory Allocator for Ind OS Userspace
//!
//! Constitutional References:
//! - REQ-USE-202: no_std-friendly alloc primitives
//! - INV-LIB-001: Safe Bounded no_std Allocation
//! - Rule N-01: Zero external crates
//! - Rule N-04: Zero unsafe code in new modules

pub const USER_HEAP_SIZE: usize = 65536; // 64 KiB default user heap pool

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    InvalidAlignment,
    BlockNotFound,
}

/// Bounded bump/block user heap allocator operating on safe pre-allocated buffer.
pub struct UserHeapAllocator {
    heap: [u8; USER_HEAP_SIZE],
    allocated_bytes: usize,
    allocation_count: usize,
}

impl UserHeapAllocator {
    pub const fn new() -> Self {
        Self {
            heap: [0u8; USER_HEAP_SIZE],
            allocated_bytes: 0,
            allocation_count: 0,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.heap[..self.allocated_bytes]
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> Result<usize, AllocError> {
        if align == 0 || (align & (align - 1)) != 0 {
            return Err(AllocError::InvalidAlignment);
        }
        let current_offset = self.allocated_bytes;
        let aligned_offset = (current_offset + (align - 1)) & !(align - 1);
        let padding = aligned_offset - current_offset;
        let total_needed = padding + size;

        if self.allocated_bytes + total_needed > USER_HEAP_SIZE {
            return Err(AllocError::OutOfMemory);
        }

        self.allocated_bytes += total_needed;
        self.allocation_count += 1;
        Ok(aligned_offset)
    }

    pub fn free(&mut self, _offset: usize, _size: usize) -> Result<(), AllocError> {
        // In simple bump allocator, frees reduce active count
        if self.allocation_count > 0 {
            self.allocation_count -= 1;
        }
        Ok(())
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }

    pub fn remaining_bytes(&self) -> usize {
        USER_HEAP_SIZE.saturating_sub(self.allocated_bytes)
    }

    pub fn active_allocations(&self) -> usize {
        self.allocation_count
    }

    pub fn reset(&mut self) {
        self.allocated_bytes = 0;
        self.allocation_count = 0;
    }
}
