//! Kernel Heap Allocator for IndK Microkernel
//! Implements core::alloc::GlobalAlloc with coalescing first-fit block management.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use crate::sync::Spinlock;

pub const DEFAULT_HEAP_SIZE: usize = 1024 * 1024; // 1 MB early kernel heap arena

#[repr(C, align(16))]
struct BlockHeader {
    size: usize,       // Usable payload size in bytes
    is_free: bool,
    next: *mut BlockHeader,
    prev: *mut BlockHeader,
}

impl BlockHeader {
    #[inline(always)]
    fn payload_ptr(&mut self) -> *mut u8 {
        unsafe { (self as *mut BlockHeader).add(1) as *mut u8 }
    }

    #[inline(always)]
    unsafe fn from_payload_ptr(ptr: *mut u8) -> *mut BlockHeader {
        (ptr as *mut BlockHeader).sub(1)
    }
}

pub struct HeapAllocatorInner {
    arena_start: usize,
    arena_size: usize,
    head: *mut BlockHeader,
    used_bytes: usize,
    alloc_count: usize,
    dealloc_count: usize,
}

unsafe impl Send for HeapAllocatorInner {}

impl HeapAllocatorInner {
    pub const fn empty() -> Self {
        Self {
            arena_start: 0,
            arena_size: 0,
            head: ptr::null_mut(),
            used_bytes: 0,
            alloc_count: 0,
            dealloc_count: 0,
        }
    }

    /// Initialize the heap with a dedicated memory arena
    pub unsafe fn init(&mut self, start: usize, size: usize) {
        // Enforce 16-byte alignment
        let aligned_start = (start + 15) & !15;
        let effective_size = size.saturating_sub(aligned_start - start);
        let header_size = core::mem::size_of::<BlockHeader>();

        if effective_size <= header_size {
            return;
        }

        self.arena_start = aligned_start;
        self.arena_size = effective_size;
        self.used_bytes = 0;
        self.alloc_count = 0;
        self.dealloc_count = 0;

        let initial_block = aligned_start as *mut BlockHeader;
        (*initial_block).size = effective_size - header_size;
        (*initial_block).is_free = true;
        (*initial_block).next = ptr::null_mut();
        (*initial_block).prev = ptr::null_mut();

        self.head = initial_block;
    }

    pub fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let _align = layout.align().max(16);
        let size = (layout.size() + 15) & !15; // 16-byte aligned size
        let header_size = core::mem::size_of::<BlockHeader>();

        let mut curr = self.head;

        while !curr.is_null() {
            unsafe {
                if (*curr).is_free && (*curr).size >= size {
                    let leftover = (*curr).size - size;

                    // Split block if remainder can fit another block header + minimal payload
                    if leftover >= header_size + 16 {
                        let new_block = (curr as *mut u8).add(header_size + size) as *mut BlockHeader;
                        (*new_block).size = leftover - header_size;
                        (*new_block).is_free = true;
                        (*new_block).next = (*curr).next;
                        (*new_block).prev = curr;

                        if !(*curr).next.is_null() {
                            (*(*curr).next).prev = new_block;
                        }

                        (*curr).next = new_block;
                        (*curr).size = size;
                    }

                    (*curr).is_free = false;
                    self.used_bytes += (*curr).size;
                    self.alloc_count += 1;
                    return (*curr).payload_ptr();
                }
                curr = (*curr).next;
            }
        }

        ptr::null_mut()
    }

    pub fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() {
            return;
        }

        unsafe {
            let header = BlockHeader::from_payload_ptr(ptr);
            if (*header).is_free {
                panic!("INV-MEM-003 VIOLATION: Double-free detected on kernel heap at 0x{:08x}!", ptr as usize);
            }

            (*header).is_free = true;
            self.used_bytes = self.used_bytes.saturating_sub((*header).size);
            self.dealloc_count += 1;

            // Coalesce with next block if free
            let next = (*header).next;
            if !next.is_null() && (*next).is_free {
                let header_size = core::mem::size_of::<BlockHeader>();
                (*header).size += header_size + (*next).size;
                (*header).next = (*next).next;
                if !(*next).next.is_null() {
                    (*(*next).next).prev = header;
                }
            }

            // Coalesce with prev block if free
            let prev = (*header).prev;
            if !prev.is_null() && (*prev).is_free {
                let header_size = core::mem::size_of::<BlockHeader>();
                (*prev).size += header_size + (*header).size;
                (*prev).next = (*header).next;
                if !(*header).next.is_null() {
                    (*(*header).next).prev = prev;
                }
            }
        }
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub fn free_bytes(&self) -> usize {
        self.arena_size.saturating_sub(self.used_bytes)
    }

    pub fn alloc_count(&self) -> usize {
        self.alloc_count
    }

    pub fn dealloc_count(&self) -> usize {
        self.dealloc_count
    }
}

/// Global thread-safe Kernel Heap Allocator
pub struct LockedHeap(Spinlock<HeapAllocatorInner>);

impl LockedHeap {
    pub const fn empty() -> Self {
        Self(Spinlock::new(HeapAllocatorInner::empty()))
    }

    pub unsafe fn init(&self, start: usize, size: usize) {
        self.0.lock().init(start, size);
    }

    pub fn stats(&self) -> (usize, usize, usize, usize) {
        let h = self.0.lock();
        (h.used_bytes(), h.free_bytes(), h.alloc_count(), h.dealloc_count())
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(ptr, layout);
    }
}
