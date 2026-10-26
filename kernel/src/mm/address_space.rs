//! Kernel & User Address-Space Layout and Per-Process AddressSpace Manager
//! Enforces higher-half split and invariants from docs/architecture/memory-spec.md
//! (INV-MEM-001 NULL trap, INV-MEM-002 PXN/UXN, INV-MEM-004 Read-Only page tables).

use core::arch::asm;
use crate::mm::frame::MemoryError;
use crate::mm::page_table::{PageTableManager, PageTableFlags, PTE_PXN, PTE_NG};

pub const USER_SPACE_START: usize    = 0x0000_0000_0040_0000; // Above 4MB NULL trap zone
pub const USER_SPACE_END: usize      = 0x0000_7FFF_FFFF_FFFF; // Lower 128 TB (TTBR0_EL1)

pub const KERNEL_SPACE_START: usize  = 0xFFFF_8000_0000_0000; // Upper 128 TB (TTBR1_EL1)
pub const PHYS_DIRECT_MAP: usize     = 0xFFFF_8000_0000_0000;
pub const KERNEL_IMAGE_BASE: usize   = 0xFFFF_8000_4000_0000;
pub const KERNEL_HEAP_BASE: usize    = 0xFFFF_FFFF_8000_0000;
pub const HIGH_FIXMAP_BASE: usize    = 0xFFFF_FFFF_FFFF_0000;

/// Enforce INV-MEM-001: Address 0x0 is strictly unmapped
#[inline(always)]
pub fn is_valid_user_address(addr: usize) -> bool {
    addr >= USER_SPACE_START && addr <= USER_SPACE_END
}

/// Check if an address falls within the higher-half kernel virtual address space
#[inline(always)]
pub fn is_kernel_address(addr: usize) -> bool {
    addr >= KERNEL_SPACE_START
}

/// Convert physical address to direct-mapped higher-half virtual address
#[inline(always)]
pub const fn phys_to_virt(phys: usize) -> usize {
    PHYS_DIRECT_MAP + phys
}

/// Convert direct-mapped higher-half virtual address to physical address
#[inline(always)]
pub const fn virt_to_phys(virt: usize) -> usize {
    virt - PHYS_DIRECT_MAP
}

/// Per-Process Virtual Address Space (governed by TTBR0_EL1)
pub struct AddressSpace {
    page_table: PageTableManager,
    asid: u16,
}

impl AddressSpace {
    /// Creates a new per-process address space with dedicated root translation table and ASID
    pub fn new(asid: u16) -> Result<Self, MemoryError> {
        let page_table = PageTableManager::create_new()?;
        Ok(Self { page_table, asid })
    }

    /// Maps a user page enforcing INV-MEM-001 (NULL trap) and INV-MEM-002 (PXN defense)
    pub fn map_user(&mut self, virt: usize, phys: usize, mut flags: PageTableFlags) -> Result<(), MemoryError> {
        // Enforce INV-MEM-001: Cannot map within 0x0..0x0040_0000 NULL trap zone or outside user space
        if !is_valid_user_address(virt) {
            return Err(MemoryError::PermissionDenied);
        }

        // Enforce INV-MEM-002: Kernel cannot execute user-space code (PXN bit) & mark Non-Global (NG)
        flags.0 |= PTE_PXN | PTE_NG;

        self.page_table.map_page(virt, phys, flags)
    }

    /// Unmaps a user virtual page
    pub fn unmap_user(&mut self, virt: usize) -> Result<(), MemoryError> {
        if !is_valid_user_address(virt) {
            return Err(MemoryError::PermissionDenied);
        }
        self.page_table.unmap_page(virt)
    }

    /// Translates a virtual address in this address space
    pub fn translate(&self, virt: usize) -> Option<(usize, PageTableFlags)> {
        self.page_table.translate(virt)
    }

    /// Resolves a Copy-on-Write page fault within this address space
    pub fn handle_cow_fault(&mut self, virt: usize) -> Result<(), MemoryError> {
        self.page_table.handle_cow_fault(virt)
    }

    /// Returns the ASID of this address space
    pub fn asid(&self) -> u16 {
        self.asid
    }

    /// Returns the physical address of the L0 root translation table
    pub fn root_table_address(&self) -> usize {
        self.page_table.root_address()
    }

    /// Activates this address space into TTBR0_EL1 with hardware ASID binding
    pub fn activate(&self) {
        let ttbr0 = ((self.asid as u64) << 48) | (self.root_table_address() as u64);
        // SAFETY: TTBR0_EL1 is loaded with verified physical L0 table address and valid ASID.
        unsafe {
            asm!(
                "dsb ishst",
                "msr ttbr0_el1, {}",
                "isb",
                in(reg) ttbr0,
                options(nostack)
            );
        }
    }
}
