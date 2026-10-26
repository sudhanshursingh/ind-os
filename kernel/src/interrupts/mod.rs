//! Interrupt & Exception Management Subsystem for IndK
//! Implements IRQ dispatch, critical section discipline, and timer tick tracking.
//! Compliant with INDOS-SPEC-IRQ-01 and machine invariants INV-INT-001..004.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::drivers::{gicv2::{GIC, SPURIOUS_IRQ}, timer::{self, TIMER_IRQ, DEFAULT_TICK_HZ}};

/// Global monotonic timer tick counter (100 Hz / 10ms per tick)
static SYSTEM_TICKS: AtomicU64 = AtomicU64::new(0);

/// Spurious interrupt counter (INV-INT-001 / INV-IRQ-003)
static SPURIOUS_INTERRUPTS: AtomicU64 = AtomicU64::new(0);

/// Checks whether IRQ interrupts are currently unmasked on this core
#[inline]
pub fn are_interrupts_enabled() -> bool {
    let daif: u64;
    // SAFETY: Reading DAIF is architecturally defined and side-effect free.
    unsafe {
        asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
    }
    // Bit 7 is the IRQ mask bit (1 = masked/disabled, 0 = unmasked/enabled)
    (daif & (1 << 7)) == 0
}

/// Unmasks IRQ interrupts on current core (msr daifclr, #2)
#[inline]
pub fn enable_interrupts() {
    // SAFETY: Modifying DAIF to clear IRQ mask enables hardware interrupt delivery.
    unsafe {
        asm!("msr daifclr, #2", options(nomem, nostack));
    }
}

/// Masks IRQ interrupts on current core (msr daifset, #2)
#[inline]
pub fn disable_interrupts() {
    // SAFETY: Modifying DAIF to set IRQ mask enters a critical section.
    unsafe {
        asm!("msr daifset, #2", options(nomem, nostack));
    }
}

/// Disables interrupts and returns the previous DAIF register state
#[inline]
pub fn disable_interrupts_save() -> u64 {
    let daif: u64;
    // SAFETY: Reading and setting DAIF atomically disables IRQs and preserves state.
    unsafe {
        asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
        asm!("msr daifset, #2", options(nomem, nostack));
    }
    daif
}

/// Restores interrupts to the previous state specified by `daif`
#[inline]
pub fn restore_interrupts(prev_daif: u64) {
    // If the IRQ mask bit (bit 7) was 0 previously, re-enable interrupts
    if (prev_daif & (1 << 7)) == 0 {
        enable_interrupts();
    }
}

/// RAII Guard for scoped critical sections (INV-INT-002)
pub struct CriticalSectionGuard {
    prev_daif: u64,
}

impl Drop for CriticalSectionGuard {
    fn drop(&mut self) {
        restore_interrupts(self.prev_daif);
    }
}

/// Executes closure `f` inside an interrupt-disabled critical section (INV-INT-002)
#[inline]
pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = CriticalSectionGuard {
        prev_daif: disable_interrupts_save(),
    };
    let result = f();
    drop(guard);
    result
}

/// Initializes GICv2, enables physical timer PPI (IRQ 30), and arms 100 Hz timer
pub fn init_interrupts() {
    // Initialize GIC distributor and CPU interface
    GIC.lock().init();

    // Enable timer PPI line (IRQ 30) in GIC
    GIC.lock().enable_irq(TIMER_IRQ);

    // Configure Generic Timer for 100 Hz ticks
    timer::init_timer(DEFAULT_TICK_HZ);
}

/// Returns the current monotonic system tick count
#[inline]
pub fn get_ticks() -> u64 {
    SYSTEM_TICKS.load(Ordering::Relaxed)
}

/// Returns the count of spurious interrupts handled
#[inline]
pub fn get_spurious_count() -> u64 {
    SPURIOUS_INTERRUPTS.load(Ordering::Relaxed)
}

/// Top-level IRQ dispatcher called from assembly `vec_curr_el_spx_irq` and `handle_lower_el_irq`
#[no_mangle]
pub extern "C" fn handle_irq_dispatch() -> bool {
    // Machine Invariant INV-INT-004: Verify execution on valid aligned kernel stack
    let sp: u64;
    // SAFETY: Reading SP register is side-effect free.
    unsafe {
        asm!("mov {}, sp", out(reg) sp, options(nomem, nostack));
    }
    assert!(sp % 16 == 0, "INV-INT-004 Violation: Interrupt stack unaligned");

    // Acknowledge interrupt at GIC CPU Interface
    let irq = GIC.lock().acknowledge_irq();

    // Machine Invariant INV-INT-001: Handle spurious interrupt ID 1023
    if irq == SPURIOUS_IRQ {
        SPURIOUS_INTERRUPTS.fetch_add(1, Ordering::Relaxed);
        return false;
    }

    // Handle timer interrupt
    if irq == TIMER_IRQ {
        // Reload countdown timer to de-assert interrupt line
        timer::reload_timer();

        // Increment monotonic tick counter
        SYSTEM_TICKS.fetch_add(1, Ordering::Relaxed);

        // Signal End of Interrupt (EOI) to GIC (INV-INT-003)
        GIC.lock().end_of_interrupt(irq);
        return true;
    }

    // Peripheral or other IRQ: signal EOI
    GIC.lock().end_of_interrupt(irq);
    false
}
