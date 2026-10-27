//! Touch Input Pipeline & Kernel IPC Optimization (SP-111, REQ-HW-107, REQ-UX-102)
//!
//! Standards: Bible §8.16; INV-TOUCH-OPT-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure safe Rust

pub const CAP_TOUCH_OPT: u64 = 0x0000_0000_0000_0100;
pub const TOUCH_OPT_MAX_BATCH: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchOptError {
    PermissionDenied,
    BatchOverflow,
    InvalidCoordinates,
    IpcStall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FastTouchPoint {
    pub x: u16,
    pub y: u16,
    pub pressure: u8,
    pub timestamp_us: u32,
}

impl FastTouchPoint {
    pub const fn new(x: u16, y: u16, pressure: u8, timestamp_us: u32) -> Self {
        Self { x, y, pressure, timestamp_us }
    }
}

pub struct TouchOptEngine {
    batch: [FastTouchPoint; TOUCH_OPT_MAX_BATCH],
    batch_len: usize,
    last_latency_us: u32,
    dispatched_events: u64,
}

impl TouchOptEngine {
    pub const fn new() -> Self {
        Self {
            batch: [FastTouchPoint::new(0, 0, 0, 0); TOUCH_OPT_MAX_BATCH],
            batch_len: 0,
            last_latency_us: 0,
            dispatched_events: 0,
        }
    }

    pub fn push_fast_point(&mut self, pt: FastTouchPoint, caps: u64) -> Result<(), TouchOptError> {
        if (caps & CAP_TOUCH_OPT) == 0 {
            return Err(TouchOptError::PermissionDenied);
        }
        if pt.x >= 1080 || pt.y >= 2460 {
            return Err(TouchOptError::InvalidCoordinates);
        }
        if self.batch_len >= TOUCH_OPT_MAX_BATCH {
            return Err(TouchOptError::BatchOverflow);
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
