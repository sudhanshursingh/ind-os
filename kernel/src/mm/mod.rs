//! Memory Management Subsystem for IndK
//! Includes DTB discovery, 4KB page frame allocator, address space layout, and kernel heap.

pub mod address_space;
pub mod dtb;
pub mod frame;
pub mod heap;

pub use address_space::*;
pub use dtb::{DtbError, DtbParser, PhysicalMemoryRegion};
pub use frame::{MemoryError, PageFrameAllocator, PhysFrame, PAGE_SIZE};
pub use heap::{LockedHeap, DEFAULT_HEAP_SIZE};

use crate::sync::Spinlock;

extern "C" {
    static __kernel_image_end: u8;
}

pub static FRAME_ALLOCATOR: Spinlock<PageFrameAllocator> = Spinlock::new(PageFrameAllocator::new());

#[global_allocator]
pub static HEAP: LockedHeap = LockedHeap::empty();

/// Initialize memory management from the Device Tree Blob
pub fn init_memory(dtb_ptr: usize) -> Result<PhysicalMemoryRegion, DtbError> {
    let parser = unsafe { DtbParser::from_raw(dtb_ptr)? };
    let dtb_size = parser.total_size();
    let mem_region = parser.discover_memory()?;

    let mut allocator = FRAME_ALLOCATOR.lock();
    allocator.init(mem_region.base, mem_region.size);

    // Reserve physical region below kernel image start (0x40000000 .. 0x40080000)
    let kernel_start = 0x40080000;
    if mem_region.base < kernel_start {
        allocator.mark_reserved(mem_region.base, kernel_start - mem_region.base);
    }

    // Reserve kernel image text, rodata, data, bss, stack
    let kernel_end = unsafe { &__kernel_image_end as *const u8 as usize };
    if kernel_end > kernel_start {
        allocator.mark_reserved(kernel_start, kernel_end - kernel_start);
    }

    // Reserve Device Tree Blob
    allocator.mark_reserved(dtb_ptr, dtb_size);

    Ok(mem_region)
}

/// Initialize the dynamic kernel heap by allocating contiguous frames
pub fn init_heap() -> usize {
    let frame_count = DEFAULT_HEAP_SIZE / PAGE_SIZE; // 256 frames (1 MB)
    let mut allocator = FRAME_ALLOCATOR.lock();

    // Allocate the first frame to anchor the heap arena
    let first_frame = allocator.alloc_frame().expect("Failed to allocate initial heap frame");
    let heap_start = first_frame.addr();

    // Allocate the remaining contiguous frames
    for i in 1..frame_count {
        let expected_addr = heap_start + (i * PAGE_SIZE);
        let f = allocator.alloc_frame().expect("Failed to allocate contiguous heap frame");
        assert_eq!(f.addr(), expected_addr, "Heap frames must be physically contiguous in early bootstrap");
    }

    // SAFETY: The frames are allocated from physical RAM and reserved exclusively for the heap arena.
    unsafe {
        HEAP.init(heap_start, DEFAULT_HEAP_SIZE);
    }

    heap_start
}
