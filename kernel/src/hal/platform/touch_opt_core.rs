//! Kernel Hardware Touch Input Optimization & Fast-Path IPC (SP-111, REQ-HW-107, REQ-UX-102)
//!
//! Standards: Bible §8.16; INV-TOUCH-OPT-001..003
//! Rule N-01: Zero external crates

pub const KERNEL_CAP_TOUCH_OPT: u64 = 0x0000_0000_0000_0100;
pub const KERNEL_TOUCH_OPT_MAX_BATCH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelTouchOptError {
    PermissionDenied,
    BatchOverflow,
    InvalidCoordinates,
    IpcStall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelFastTouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressure: u8,
    pub timestamp_us: u32,
}

impl KernelFastTouchPoint {
    pub const fn new(x: u16, y: u16, pressure: u8, timestamp_us: u32) -> Self {
        Self { x, y, pressure, timestamp_us }
    }
}

pub struct KernelTouchOptEngine {
    batch: [KernelFastTouchPoint; KERNEL_TOUCH_OPT_MAX_BATCH],
    batch_len: usize,
    last_latency_us: u32,
    dispatched_events: u64,
}

impl KernelTouchOptEngine {
    pub const fn new() -> Self {
        Self {
            batch: [KernelFastTouchPoint::new(0, 0, 0, 0); KERNEL_TOUCH_OPT_MAX_BATCH],
            batch_len: 0,
            last_latency_us: 0,
            dispatched_events: 0,
        }
    }

    pub fn push_fast_point(&mut self, pt: KernelFastTouchPoint, caps: u64) -> Result<(), KernelTouchOptError> {
        if (caps & KERNEL_CAP_TOUCH_OPT) == 0 {
            return Err(KernelTouchOptError::PermissionDenied);
        }
        if pt.x >= 1080 || pt.y >= 2460 {
            return Err(KernelTouchOptError::InvalidCoordinates);
        }
        if self.batch_len >= KERNEL_TOUCH_OPT_MAX_BATCH {
            return Err(KernelTouchOptError::BatchOverflow);
        }
        self.batch[self.batch_len] = pt;
        self.batch_len += 1;
        self.dispatched_events += 1;
        Ok(())
    }

    pub fn flush_batch(&mut self, now_us: u32) -> (usize, u32) {
        let count = self.batch_len;
        if count > 0 {
            let start = self.batch[0].timestamp_us;
            self.last_latency_us = if now_us > start { now_us - start } else { 0 };
        }
        self.batch_len = 0;
        (count, self.last_latency_us)
    }

    pub fn last_latency_us(&self) -> u32 {
        self.last_latency_us
    }

    pub fn dispatched_events(&self) -> u64 {
        self.dispatched_events
    }
}
