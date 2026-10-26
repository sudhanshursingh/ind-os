//! Kernel & User Address-Space Layout Architecture
//! Enforces higher-half split and invariants from docs/architecture/memory-spec.md

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
