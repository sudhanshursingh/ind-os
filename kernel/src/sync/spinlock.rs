//! Freestanding Spinlock for IndK Microkernel
//! Implements mutual exclusion and IRQ-safe spinlocks without heap allocation.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

#[repr(C)]
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: Spinlock provides synchronized mutual exclusion across threads/cores.
unsafe impl<T: Send> Sync for Spinlock<T> {}
// SAFETY: Spinlock provides synchronized mutual exclusion across threads/cores.
unsafe impl<T: Send> Send for Spinlock<T> {}

impl<T> Spinlock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, T> {
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        SpinlockGuard { lock: self }
    }

    pub fn try_lock(&self) -> Option<SpinlockGuard<'_, T>> {
        if self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_ok() {
            Some(SpinlockGuard { lock: self })
        } else {
            None
        }
    }

    /// Acquires lock while disabling interrupts on the current core (INV-INT-002)
    pub fn lock_irqsave(&self) -> IrqSafeSpinlockGuard<'_, T> {
        let prev_daif = crate::interrupts::disable_interrupts_save();
        while self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        IrqSafeSpinlockGuard {
            lock: self,
            prev_daif,
        }
    }
}

pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: Lock is held via atomic acquire barrier.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Lock is held exclusively via atomic acquire barrier.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

/// Guard that restores previous interrupt mask state when released (INV-INT-002)
pub struct IrqSafeSpinlockGuard<'a, T> {
    lock: &'a Spinlock<T>,
    prev_daif: u64,
}

impl<T> Deref for IrqSafeSpinlockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: Lock is held exclusively and interrupts are disabled.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for IrqSafeSpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Lock is held exclusively and interrupts are disabled.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for IrqSafeSpinlockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        crate::interrupts::restore_interrupts(self.prev_daif);
    }
}
