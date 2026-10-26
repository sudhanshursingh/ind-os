//! ARM Generic Physical Timer Driver for IndK
//! Configures CNTP_*_EL0 system registers for periodic quantum ticks (INV-IRQ-004).

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

/// Non-secure EL1 Physical Timer PPI IRQ Number
pub const TIMER_IRQ: u32 = 30;

/// Default scheduler tick rate: 100 Hz (10ms quantum)
pub const DEFAULT_TICK_HZ: u64 = 100;

static TIMER_INTERVAL: AtomicU64 = AtomicU64::new(0);

/// Reads the counter-timer frequency register (CNTFRQ_EL0) in Hz
#[inline]
pub fn read_frequency() -> u64 {
    let freq: u64;
    // SAFETY: Reading CNTFRQ_EL0 is architecturally defined and side-effect free.
    unsafe {
        asm!("mrs {}, cntfrq_el0", out(reg) freq, options(nomem, nostack));
    }
    freq
}

/// Reads the 64-bit physical count register (CNTPCT_EL0)
#[inline]
pub fn read_count() -> u64 {
    let count: u64;
    // SAFETY: Reading CNTPCT_EL0 is architecturally defined and side-effect free.
    unsafe {
        asm!("mrs {}, cntpct_el0", out(reg) count, options(nomem, nostack));
    }
    count
}

/// Configures the ARM Generic Timer for periodic ticks at `hz` frequency
pub fn init_timer(hz: u64) {
    let freq = read_frequency();
    let interval = freq / hz;
    TIMER_INTERVAL.store(interval, Ordering::Release);

    // Disable timer before reprogramming
    // SAFETY: Disabling CNTP_CTL_EL0 ensures atomic reload of timer parameters.
    unsafe {
        asm!("msr cntp_ctl_el0, {}", in(reg) 0u64, options(nomem, nostack));
    }

    // Set initial down-counter value
    // SAFETY: Loading CNTP_TVAL_EL0 programs the compare value relative to CNTPCT_EL0.
    unsafe {
        asm!("msr cntp_tval_el0, {}", in(reg) interval, options(nomem, nostack));
    }

    // Enable timer with interrupt unmasked (ENABLE=1, IMASK=0)
    // SAFETY: Enabling physical timer activates IRQ 30 assertion on countdown expiry.
    unsafe {
        asm!("msr cntp_ctl_el0, {}", in(reg) 1u64, options(nomem, nostack));
    }
}

/// Reloads the down-counter with the configured quantum interval
#[inline]
pub fn reload_timer() {
    let interval = TIMER_INTERVAL.load(Ordering::Relaxed);
    // SAFETY: Writing CNTP_TVAL_EL0 resets countdown and de-asserts timer interrupt line.
    unsafe {
        asm!("msr cntp_tval_el0, {}", in(reg) interval, options(nomem, nostack));
    }
}

/// Disables the physical timer
pub fn disable_timer() {
    // SAFETY: Clearing CNTP_CTL_EL0 deactivates timer and stops IRQ 30 generation.
    unsafe {
        asm!("msr cntp_ctl_el0, {}", in(reg) 0u64, options(nomem, nostack));
    }
}
