//! ARM64 Architectural Subsystem for IndK
//! Implements exception handling context and global assembly inclusion.

use core::arch::global_asm;

// Embed boot entry and exception vectors
global_asm!(include_str!("boot.s"));
global_asm!(include_str!("exceptions.s"));

/// CPU Context preserved by `exceptions.s` on exception entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExceptionContext {
    pub regs: [u64; 30],      // x0..x29
    pub lr: u64,              // x30 (link register)
    pub sp: u64,              // original stack pointer
    pub elr_el1: u64,         // fault instruction address
    pub spsr_el1: u64,        // processor state
    pub esr_el1: u64,         // exception syndrome register
    pub far_el1: u64,         // fault address register
    pub exception_id: u64,    // vector identification code
    pub _reserved: u64,       // 16-byte alignment padding
}

/// Early exception handler dispatched from vector table
#[no_mangle]
pub extern "C" fn handle_early_exception(ctx: &ExceptionContext) -> ! {
    crate::panic::panic_exception(ctx);
}
