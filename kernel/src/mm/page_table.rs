//! AArch64 4-Level Page Table Subsystem for IndK
//! Implements 48-bit Virtual Addressing with 4KB Granule per INDOS-SPEC-MEM-01.
//! Enforces invariants INV-MEM-001 (NULL trap), INV-MEM-002 (PXN/UXN), INV-MEM-003, INV-MEM-004.

use core::arch::asm;
use crate::mm::frame::{MemoryError, PAGE_SIZE};
use crate::mm::FRAME_ALLOCATOR;

/// Page Table Entry Descriptor Masks and Flags
pub const PTE_VALID: u64        = 1 << 0;  // Entry is present and valid
pub const PTE_TABLE: u64        = 1 << 1;  // Table descriptor at L0..L2
pub const PTE_PAGE: u64         = 1 << 1;  // Page descriptor at L3

// Memory Attributes (MAIR_EL1 indices)
pub const PTE_ATTR_DEVICE: u64  = 0 << 2;  // Index 0: Device-nGnRE MMIO
pub const PTE_ATTR_NORMAL: u64  = 1 << 2;  // Index 1: Normal Cacheable RAM

pub const PTE_NS: u64           = 1 << 5;  // Non-secure physical mapping
pub const PTE_AP_RW_KERNEL: u64 = 0b00 << 6; // Kernel Read/Write, User No Access
pub const PTE_AP_RW_USER: u64   = 0b01 << 6; // User Read/Write, Kernel Read/Write
pub const PTE_AP_RO_KERNEL: u64 = 0b10 << 6; // Kernel Read-Only, User No Access
pub const PTE_AP_RO_USER: u64   = 0b11 << 6; // User Read-Only, Kernel Read-Only

pub const PTE_SH_INNER: u64     = 0b11 << 8; // Inner Shareable
pub const PTE_AF: u64           = 1 << 10; // Access Flag (avoids MMU fault)
pub const PTE_NG: u64           = 1 << 11; // Non-Global (User ASID-bound)
pub const PTE_ADDR_MASK: u64    = 0x0000_FFFF_FFFF_F000; // Physical address mask

// Architecture Security Flags (INV-MEM-002)
pub const PTE_PXN: u64          = 1 << 53; // Privileged Execute Never (Kernel cannot execute user)
pub const PTE_UXN: u64          = 1 << 54; // Unprivileged Execute Never (User cannot execute kernel)

// Software-Defined Flags (bits 55..58 ignored by hardware)
pub const PTE_COW: u64          = 1 << 55; // Copy-on-Write tracking flag

/// Strongly typed page table permission and attribute flags
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageTableFlags(pub u64);

impl PageTableFlags {
    pub const KERNEL_CODE: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_NORMAL | PTE_AP_RO_KERNEL | PTE_SH_INNER | PTE_AF | PTE_UXN);
    pub const KERNEL_DATA: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_NORMAL | PTE_AP_RW_KERNEL | PTE_SH_INNER | PTE_AF | PTE_PXN | PTE_UXN);
    pub const KERNEL_RODATA: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_NORMAL | PTE_AP_RO_KERNEL | PTE_SH_INNER | PTE_AF | PTE_PXN | PTE_UXN);
    pub const DEVICE_MMIO: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_DEVICE | PTE_AP_RW_KERNEL | PTE_SH_INNER | PTE_AF | PTE_PXN | PTE_UXN);
    pub const USER_CODE: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_NORMAL | PTE_AP_RO_USER | PTE_SH_INNER | PTE_AF | PTE_NG | PTE_PXN);
    pub const USER_DATA: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_NORMAL | PTE_AP_RW_USER | PTE_SH_INNER | PTE_AF | PTE_NG | PTE_PXN | PTE_UXN);
    pub const USER_COW: Self = Self(PTE_VALID | PTE_PAGE | PTE_ATTR_NORMAL | PTE_AP_RO_USER | PTE_SH_INNER | PTE_AF | PTE_NG | PTE_PXN | PTE_UXN | PTE_COW);

    #[inline]
    pub fn contains(&self, bit: u64) -> bool {
        (self.0 & bit) == bit
    }

    #[inline]
    pub fn is_cow(&self) -> bool {
        self.contains(PTE_COW)
    }

    #[inline]
    pub fn is_writable(&self) -> bool {
        (self.0 & (0b10 << 6)) == 0
    }

    #[inline]
    pub fn is_user(&self) -> bool {
        (self.0 & (0b01 << 6)) != 0
    }
}

/// AArch64 Page Table Entry
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        (self.0 & PTE_VALID) != 0
    }

    #[inline]
    pub fn is_table(&self) -> bool {
        self.is_valid() && ((self.0 & PTE_TABLE) != 0)
    }

    #[inline]
    pub fn is_cow(&self) -> bool {
        (self.0 & PTE_COW) != 0
    }

    #[inline]
    pub fn phys_addr(&self) -> usize {
        (self.0 & PTE_ADDR_MASK) as usize
    }

    #[inline]
    pub fn flags(&self) -> PageTableFlags {
        PageTableFlags(self.0 & !PTE_ADDR_MASK)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    #[inline]
    pub fn set_table(&mut self, table_phys: usize) {
        self.0 = (table_phys as u64 & PTE_ADDR_MASK) | PTE_VALID | PTE_TABLE | PTE_AF;
    }

    #[inline]
    pub fn set_page(&mut self, page_phys: usize, flags: PageTableFlags) {
        self.0 = (page_phys as u64 & PTE_ADDR_MASK) | flags.0 | PTE_VALID | PTE_PAGE | PTE_AF;
    }
}

/// AArch64 4KB Page Table containing 512 entries
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::empty(); 512],
        }
    }

    pub fn clear(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.clear();
        }
    }
}

/// 4-Level Page Table Manager
pub struct PageTableManager {
    root_phys: usize,
}

impl PageTableManager {
    /// Creates a manager anchored at an existing physical L0 page table
    pub const fn from_root(root_phys: usize) -> Self {
        Self { root_phys }
    }

    /// Allocates a fresh L0 page table frame and initializes a new manager
    pub fn create_new() -> Result<Self, MemoryError> {
        let mut allocator = FRAME_ALLOCATOR.lock();
        let frame = allocator.alloc_frame()?;
        let phys = frame.addr();

        // SAFETY: The allocated frame is dedicated to the new page table and zeroed.
        unsafe {
            let ptr = phys as *mut PageTable;
            ptr.write(PageTable::new());
        }

        Ok(Self { root_phys: phys })
    }

    pub fn root_address(&self) -> usize {
        self.root_phys
    }

    /// Maps a 4KB virtual page to a physical frame with specified attributes
    pub fn map_page(&mut self, virt: usize, phys: usize, flags: PageTableFlags) -> Result<(), MemoryError> {
        assert!(virt % PAGE_SIZE == 0, "Virtual address must be 4KB aligned");
        assert!(phys % PAGE_SIZE == 0, "Physical address must be 4KB aligned");

        let l0_idx = (virt >> 39) & 0x1FF;
        let l1_idx = (virt >> 30) & 0x1FF;
        let l2_idx = (virt >> 21) & 0x1FF;
        let l3_idx = (virt >> 12) & 0x1FF;

        // Traverse or allocate L0 -> L1
        let l0_table = Self::get_table_mut(self.root_phys);
        let l1_phys = Self::get_or_create_next_level(&mut l0_table.entries[l0_idx])?;

        // Traverse or allocate L1 -> L2
        let l1_table = Self::get_table_mut(l1_phys);
        let l2_phys = Self::get_or_create_next_level(&mut l1_table.entries[l1_idx])?;

        // Traverse or allocate L2 -> L3
        let l2_table = Self::get_table_mut(l2_phys);
        let l3_phys = Self::get_or_create_next_level(&mut l2_table.entries[l2_idx])?;

        // Populate L3 Page Descriptor
        let l3_table = Self::get_table_mut(l3_phys);
        l3_table.entries[l3_idx].set_page(phys, flags);

        // Invalidate TLB for mapped virtual address
        self.invalidate_tlb_page(virt);

        Ok(())
    }

    /// Unmaps a 4KB virtual page
    pub fn unmap_page(&mut self, virt: usize) -> Result<(), MemoryError> {
        assert!(virt % PAGE_SIZE == 0, "Virtual address must be 4KB aligned");

        let l0_idx = (virt >> 39) & 0x1FF;
        let l1_idx = (virt >> 30) & 0x1FF;
        let l2_idx = (virt >> 21) & 0x1FF;
        let l3_idx = (virt >> 12) & 0x1FF;

        let l0_table = Self::get_table_mut(self.root_phys);
        if !l0_table.entries[l0_idx].is_valid() {
            return Err(MemoryError::PageNotMapped);
        }
        let l1_phys = l0_table.entries[l0_idx].phys_addr();

        let l1_table = Self::get_table_mut(l1_phys);
        if !l1_table.entries[l1_idx].is_valid() {
            return Err(MemoryError::PageNotMapped);
        }
        let l2_phys = l1_table.entries[l1_idx].phys_addr();

        let l2_table = Self::get_table_mut(l2_phys);
        if !l2_table.entries[l2_idx].is_valid() {
            return Err(MemoryError::PageNotMapped);
        }
        let l3_phys = l2_table.entries[l2_idx].phys_addr();

        let l3_table = Self::get_table_mut(l3_phys);
        if !l3_table.entries[l3_idx].is_valid() {
            return Err(MemoryError::PageNotMapped);
        }

        l3_table.entries[l3_idx].clear();
        self.invalidate_tlb_page(virt);

        Ok(())
    }

    /// Translates a virtual address to its mapped physical address and flags
    pub fn translate(&self, virt: usize) -> Option<(usize, PageTableFlags)> {
        let l0_idx = (virt >> 39) & 0x1FF;
        let l1_idx = (virt >> 30) & 0x1FF;
        let l2_idx = (virt >> 21) & 0x1FF;
        let l3_idx = (virt >> 12) & 0x1FF;
        let page_offset = virt & (PAGE_SIZE - 1);

        let l0_table = Self::get_table(self.root_phys);
        if !l0_table.entries[l0_idx].is_valid() {
            return None;
        }

        let l1_table = Self::get_table(l0_table.entries[l0_idx].phys_addr());
        if !l1_table.entries[l1_idx].is_valid() {
            return None;
        }

        let l2_table = Self::get_table(l1_table.entries[l1_idx].phys_addr());
        if !l2_table.entries[l2_idx].is_valid() {
            return None;
        }

        let l3_table = Self::get_table(l2_table.entries[l2_idx].phys_addr());
        let entry = l3_table.entries[l3_idx];
        if !entry.is_valid() {
            return None;
        }

        Some((entry.phys_addr() + page_offset, entry.flags()))
    }

    /// Handles a Copy-on-Write (CoW) page fault by allocating a private physical copy
    pub fn handle_cow_fault(&mut self, virt: usize) -> Result<(), MemoryError> {
        let page_virt = virt & !(PAGE_SIZE - 1);
        let (old_phys, flags) = self.translate(page_virt).ok_or(MemoryError::PageNotMapped)?;

        if !flags.is_cow() {
            return Err(MemoryError::PermissionDenied);
        }

        // Allocate a new physical frame for the private copy
        let mut allocator = FRAME_ALLOCATOR.lock();
        let new_frame = allocator.alloc_frame()?;
        let new_phys = new_frame.addr();
        drop(allocator);

        // Copy 4KB page content from old to new frame
        // SAFETY: Both old_phys and new_phys are verified valid 4KB page frames.
        unsafe {
            let src = (old_phys & !(PAGE_SIZE - 1)) as *const u8;
            let dst = new_phys as *mut u8;
            core::ptr::copy_nonoverlapping(src, dst, PAGE_SIZE);
        }

        // Remap virtual page with Read/Write permissions and clear COW bit
        let new_flags = PageTableFlags(
            (flags.0 & !PTE_COW & !(0b10 << 6)) | PTE_AP_RW_USER
        );
        self.map_page(page_virt, new_phys, new_flags)?;

        Ok(())
    }

    fn get_or_create_next_level(entry: &mut PageTableEntry) -> Result<usize, MemoryError> {
        if entry.is_valid() {
            Ok(entry.phys_addr())
        } else {
            let mut allocator = FRAME_ALLOCATOR.lock();
            let frame = allocator.alloc_frame()?;
            let phys = frame.addr();

            // SAFETY: Newly allocated page table is zeroed to initialize all entries as invalid.
            unsafe {
                let ptr = phys as *mut PageTable;
                ptr.write(PageTable::new());
            }

            entry.set_table(phys);
            Ok(phys)
        }
    }

    #[inline]
    fn get_table(phys: usize) -> &'static PageTable {
        // SAFETY: phys points to an identity/direct-mapped valid PageTable frame.
        unsafe { &*(phys as *const PageTable) }
    }

    #[inline]
    fn get_table_mut(phys: usize) -> &'static mut PageTable {
        // SAFETY: phys points to an identity/direct-mapped valid PageTable frame.
        unsafe { &mut *(phys as *mut PageTable) }
    }

    #[inline]
    fn invalidate_tlb_page(&self, virt: usize) {
        let va = virt >> 12;
        // SAFETY: Flushing TLB entry ensures MMU uses updated translation descriptor.
        unsafe {
            asm!(
                "dsb ishst",
                "tlbi vaae1is, {}",
                "dsb ish",
                "isb",
                in(reg) va,
                options(nostack)
            );
        }
    }
}
