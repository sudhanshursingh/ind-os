//! 4 KB Page Frame Allocator for IndK
//! Implements INV-MEM-001 and INV-MEM-003 from docs/architecture/memory-spec.md

pub const PAGE_SIZE: usize = 4096;
const MAX_FRAMES: usize = 262_144; // Manages up to 1 GB RAM (262,144 * 4KB)
const BITMAP_WORDS: usize = MAX_FRAMES / 64; // 4,096 u64 words = 32 KB

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysFrame(pub usize);

impl PhysFrame {
    #[inline(always)]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr & !(PAGE_SIZE - 1))
    }

    #[inline(always)]
    pub const fn addr(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    OutOfMemory,
    InvalidFrameAddress(usize),
    DoubleFree(usize),
    FrameNotAllocated(usize),
    PageNotMapped,
    PermissionDenied,
}

/// Early Page Frame Allocator using a flat bitmask
pub struct PageFrameAllocator {
    ram_base: usize,
    ram_size: usize,
    total_frames: usize,
    allocated_count: usize,
    bitmap: [u64; BITMAP_WORDS],
}

impl PageFrameAllocator {
    pub const fn new() -> Self {
        Self {
            ram_base: 0,
            ram_size: 0,
            total_frames: 0,
            allocated_count: 0,
            bitmap: [!0u64; BITMAP_WORDS], // Initially marked unavailable until init
        }
    }

    /// Initialize the allocator with discovered physical RAM topology
    pub fn init(&mut self, ram_base: usize, ram_size: usize) {
        self.ram_base = ram_base;
        self.ram_size = ram_size;
        self.total_frames = core::cmp::min(ram_size / PAGE_SIZE, MAX_FRAMES);
        self.allocated_count = 0;

        // Clear bits for all usable frames within RAM bounds
        let full_words = self.total_frames / 64;
        let remainder = self.total_frames % 64;

        for i in 0..full_words {
            self.bitmap[i] = 0;
        }
        if remainder > 0 {
            // Unused bits in the partial word marked 1 (unavailable)
            self.bitmap[full_words] = !0u64 << remainder;
        }
        for i in (full_words + if remainder > 0 { 1 } else { 0 })..BITMAP_WORDS {
            self.bitmap[i] = !0u64;
        }
    }

    #[inline(always)]
    fn addr_to_frame_idx(&self, addr: usize) -> Option<usize> {
        if addr < self.ram_base || addr >= self.ram_base + (self.total_frames * PAGE_SIZE) {
            None
        } else {
            Some((addr - self.ram_base) / PAGE_SIZE)
        }
    }

    /// Mark an explicit physical memory range as reserved/unavailable
    pub fn mark_reserved(&mut self, start_addr: usize, size: usize) {
        if size == 0 {
            return;
        }
        let end_addr = start_addr + size;
        let start_idx = self.addr_to_frame_idx(start_addr).unwrap_or(0);
        let end_idx = self.addr_to_frame_idx(end_addr + PAGE_SIZE - 1).unwrap_or(self.total_frames);

        for idx in start_idx..end_idx {
            let word = idx / 64;
            let bit = idx % 64;
            if (self.bitmap[word] & (1 << bit)) == 0 {
                self.bitmap[word] |= 1 << bit;
                self.allocated_count += 1;
            }
        }
    }

    /// Allocate a single 4 KB physical frame
    pub fn alloc_frame(&mut self) -> Result<PhysFrame, MemoryError> {
        for word_idx in 0..BITMAP_WORDS {
            if self.bitmap[word_idx] != !0u64 {
                let free_bit = self.bitmap[word_idx].trailing_ones() as usize;
                let frame_idx = (word_idx * 64) + free_bit;
                if frame_idx >= self.total_frames {
                    return Err(MemoryError::OutOfMemory);
                }

                // Mark frame allocated
                self.bitmap[word_idx] |= 1 << free_bit;
                self.allocated_count += 1;

                let phys_addr = self.ram_base + (frame_idx * PAGE_SIZE);
                return Ok(PhysFrame(phys_addr));
            }
        }

        Err(MemoryError::OutOfMemory)
    }

    /// Free an allocated physical frame
    /// INV-MEM-003: Double-free or freeing an unallocated frame triggers an immediate panic
    pub fn free_frame(&mut self, frame: PhysFrame) -> Result<(), MemoryError> {
        let addr = frame.addr();
        let frame_idx = match self.addr_to_frame_idx(addr) {
            Some(idx) => idx,
            None => return Err(MemoryError::InvalidFrameAddress(addr)),
        };

        let word = frame_idx / 64;
        let bit = frame_idx % 64;

        if (self.bitmap[word] & (1 << bit)) == 0 {
            // INV-MEM-003 Violation!
            panic!("INV-MEM-003 VIOLATION: Double-free or freeing unallocated physical frame at 0x{:08x}!", addr);
        }

        // Mark frame free
        self.bitmap[word] &= !(1 << bit);
        self.allocated_count -= 1;

        Ok(())
    }

    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    pub fn allocated_frames(&self) -> usize {
        self.allocated_count
    }

    pub fn free_frames(&self) -> usize {
        self.total_frames.saturating_sub(self.allocated_count)
    }

    pub fn is_frame_allocated(&self, frame: PhysFrame) -> bool {
        if let Some(idx) = self.addr_to_frame_idx(frame.addr()) {
            let word = idx / 64;
            let bit = idx % 64;
            (self.bitmap[word] & (1 << bit)) != 0
        } else {
            true
        }
    }
}
