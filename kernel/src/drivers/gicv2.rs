//! ARM Generic Interrupt Controller v2 (GICv2) Driver for IndK
//! Implements GIC Distributor (GICD) and CPU Interface (GICC) per INDOS-SPEC-IRQ-01.

use core::ptr::{read_volatile, write_volatile};
use crate::sync::Spinlock;

/// QEMU virt platform GICv2 MMIO base addresses
pub const GICD_BASE: usize = 0x0800_0000;
pub const GICC_BASE: usize = 0x0801_0000;

/// GIC Distributor Register Offsets
pub const GICD_CTLR: usize = 0x000;
pub const GICD_TYPER: usize = 0x004;
pub const GICD_ISENABLER: usize = 0x100;
pub const GICD_ICENABLER: usize = 0x180;
pub const GICD_ISPENDR: usize = 0x200;
pub const GICD_ICPENDR: usize = 0x280;
pub const GICD_IPRIORITYR: usize = 0x400;
pub const GICD_ITARGETSR: usize = 0x800;
pub const GICD_ICFGR: usize = 0xC00;

/// GIC CPU Interface Register Offsets
pub const GICC_CTLR: usize = 0x000;
pub const GICC_PMR: usize = 0x004;
pub const GICC_BPR: usize = 0x008;
pub const GICC_IAR: usize = 0x00C;
pub const GICC_EOIR: usize = 0x010;

/// Spurious Interrupt ID per ARM GICv2 Architecture Specification
pub const SPURIOUS_IRQ: u32 = 1023;

/// ARM GICv2 Controller instance
pub struct GicV2 {
    dist_base: usize,
    cpu_base: usize,
}

impl GicV2 {
    /// Creates a new uninitialized GICv2 instance
    pub const fn new(dist_base: usize, cpu_base: usize) -> Self {
        Self {
            dist_base,
            cpu_base,
        }
    }

    #[inline]
    fn dist_read32(&self, offset: usize) -> u32 {
        // SAFETY: dist_base is verified MMIO page mapped for GICD.
        unsafe { read_volatile((self.dist_base + offset) as *const u32) }
    }

    #[inline]
    fn dist_write32(&self, offset: usize, val: u32) {
        // SAFETY: dist_base is verified MMIO page mapped for GICD.
        unsafe { write_volatile((self.dist_base + offset) as *mut u32, val) }
    }

    #[inline]
    fn dist_write8(&self, offset: usize, val: u8) {
        // SAFETY: dist_base is verified MMIO page mapped for GICD.
        unsafe { write_volatile((self.dist_base + offset) as *mut u8, val) }
    }

    #[inline]
    fn cpu_read32(&self, offset: usize) -> u32 {
        // SAFETY: cpu_base is verified MMIO page mapped for GICC.
        unsafe { read_volatile((self.cpu_base + offset) as *const u32) }
    }

    #[inline]
    fn cpu_write32(&self, offset: usize, val: u32) {
        // SAFETY: cpu_base is verified MMIO page mapped for GICC.
        unsafe { write_volatile((self.cpu_base + offset) as *mut u32, val) }
    }

    /// Initializes both GICD and GICC on CPU core 0
    pub fn init(&mut self) {
        // 1. Disable Distributor and CPU Interface during initialization
        self.dist_write32(GICD_CTLR, 0);
        self.cpu_write32(GICC_CTLR, 0);

        // 2. Discover total number of supported interrupt lines
        let typer = self.dist_read32(GICD_TYPER);
        let it_lines = (typer & 0x1F) as usize;
        let max_irqs = (it_lines + 1) * 32;

        // 3. Disable all interrupts and clear pending states
        for i in 0..=(max_irqs / 32) {
            self.dist_write32(GICD_ICENABLER + (i * 4), 0xFFFF_FFFF);
            self.dist_write32(GICD_ICPENDR + (i * 4), 0xFFFF_FFFF);
        }

        // 4. Set default priority (0xA0) for all interrupts
        for i in 0..max_irqs {
            self.dist_write8(GICD_IPRIORITYR + i, 0xA0);
        }

        // 5. Route all SPIs (32..max_irqs) to CPU 0 (target bitmask 0x01)
        for i in 32..max_irqs {
            self.dist_write8(GICD_ITARGETSR + i, 0x01);
        }

        // 6. Set CPU Interface priority mask to accept all priorities (0xFF)
        self.cpu_write32(GICC_PMR, 0xFF);
        self.cpu_write32(GICC_BPR, 0x00);

        // 7. Enable CPU Interface and Distributor
        self.cpu_write32(GICC_CTLR, 1);
        self.dist_write32(GICD_CTLR, 1);
    }

    /// Enables an interrupt line by IRQ ID
    pub fn enable_irq(&self, irq: u32) {
        if irq >= 32 {
            // Route SPI to CPU 0
            self.dist_write8(GICD_ITARGETSR + (irq as usize), 0x01);
        }
        // Set priority (0x80 = higher priority than default 0xA0)
        self.dist_write8(GICD_IPRIORITYR + (irq as usize), 0x80);

        // Set enable bit in GICD_ISENABLER
        let reg_offset = GICD_ISENABLER + ((irq / 32) as usize * 4);
        let bit = 1u32 << (irq % 32);
        self.dist_write32(reg_offset, bit);
    }

    /// Disables an interrupt line by IRQ ID
    pub fn disable_irq(&self, irq: u32) {
        let reg_offset = GICD_ICENABLER + ((irq / 32) as usize * 4);
        let bit = 1u32 << (irq % 32);
        self.dist_write32(reg_offset, bit);
    }

    /// Reads Interrupt Acknowledge Register (GICC_IAR)
    pub fn acknowledge_irq(&self) -> u32 {
        self.cpu_read32(GICC_IAR) & 0x3FF
    }

    /// Signals End of Interrupt (GICC_EOIR)
    pub fn end_of_interrupt(&self, irq: u32) {
        self.cpu_write32(GICC_EOIR, irq);
    }
}

/// Global GICv2 instance protected by Spinlock
pub static GIC: Spinlock<GicV2> = Spinlock::new(GicV2::new(GICD_BASE, GICC_BASE));
