//! ARM64 Architectural Subsystem for IndK
//! Implements exception handling context and global assembly inclusion.

use core::arch::global_asm;

// Embed boot entry, exception vectors, and user mode traps
global_asm!(include_str!("boot.s"));
global_asm!(include_str!("exceptions.s"));
global_asm!(include_str!("user_trap.s"));

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

/// Lower EL synchronous exception handler (SVC #0 system calls and faults)
#[no_mangle]
pub extern "C" fn handle_lower_el_sync(ctx: &mut crate::task::UserContext) {
    let ec = (ctx.esr_el1 >> 26) & 0x3F;
    if ec == 0x15 {
        // SVC instruction in AArch64 state
        crate::syscall::handle_syscall(ctx);
    } else {
        crate::panic::EarlyUart::write_str("\n[LOWER_EL_SYNC] Unhandled Lower EL Exception!\n");
        crate::panic::EarlyUart::write_str("  EC:  ");
        crate::panic::EarlyUart::write_hex_u64(ec);
        crate::panic::EarlyUart::write_str("\n  ESR: ");
        crate::panic::EarlyUart::write_hex_u64(ctx.esr_el1);
        crate::panic::EarlyUart::write_str("\n  ELR: ");
        crate::panic::EarlyUart::write_hex_u64(ctx.elr_el1);
        crate::panic::EarlyUart::write_str("\n");
        panic!("Unhandled lower EL exception");
    }
}

/// Lower EL IRQ handler (hardware interrupts arriving during EL0 execution)
#[no_mangle]
pub extern "C" fn handle_lower_el_irq(ctx: &mut crate::task::UserContext) {
    let is_timer = crate::interrupts::handle_irq_dispatch();
    if is_timer {
        let reschedule = crate::task::scheduler::SCHEDULER.lock().on_timer_tick();
        if reschedule {
            let pid = crate::task::scheduler::SCHEDULER.lock().current_process().map(|p| p.pid).unwrap_or(0);
            crate::log_info!("sched", "Time quantum expired for PID {}: preemptive context switch triggered", pid);
            crate::task::scheduler::SCHEDULER.lock().schedule_next(ctx);
        }
    }
}
