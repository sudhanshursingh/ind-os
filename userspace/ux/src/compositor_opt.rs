//! Display Compositor 120Hz Refresh Pipelining (SP-113, REQ-HW-106, REQ-PERF-104)
//!
//! Standards: Bible §8.16; INV-COMP-OPT-001..003
//! Rule N-01: Zero external crates
//! Rule N-04: Pure safe Rust

pub const CAP_COMPOSITOR_OPT: u64 = 0x0000_0000_0000_0400;
pub const TARGET_FRAME_TIME_US: u32 = 8333; // 8.333ms (120Hz)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorOptError {
    PermissionDenied,
    FrameDropped,
    BufferMismatch,
    InvalidDamageRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameTimingSample {
    pub frame_idx: u32,
    pub render_time_us: u32,
    pub swap_time_us: u32,
    pub dirty_pixels: u32,
}

impl FrameTimingSample {
    pub const fn new(frame_idx: u32, render_time_us: u32, swap_time_us: u32, dirty_pixels: u32) -> Self {
        Self { frame_idx, render_time_us, swap_time_us, dirty_pixels }
    }
}

pub struct CompositorOptEngine {
    total_frames: u32,
    dropped_frames: u32,
    p95_frame_time_us: u32,
    max_frame_time_us: u32,
}

impl CompositorOptEngine {
    pub const fn new() -> Self {
        Self {
            total_frames: 0,
            dropped_frames: 0,
            p95_frame_time_us: 0,
            max_frame_time_us: 0,
        }
    }

    pub fn record_frame(&mut self, timing: FrameTimingSample, caps: u64) -> Result<(), CompositorOptError> {
        if (caps & CAP_COMPOSITOR_OPT) == 0 {
            return Err(CompositorOptError::PermissionDenied);
        }
        let total_us = timing.render_time_us + timing.swap_time_us;
        if total_us > TARGET_FRAME_TIME_US {
            self.dropped_frames += 1;
        }
        if total_us > self.max_frame_time_us {
            self.max_frame_time_us = total_us;
        }
        self.p95_frame_time_us = (self.p95_frame_time_us * 95 + total_us * 5) / 100;
        self.total_frames += 1;
        Ok(())
    }

    pub fn total_frames(&self) -> u32 {
        self.total_frames
    }

    pub fn dropped_frames(&self) -> u32 {
        self.dropped_frames
    }

    pub fn p95_frame_time_us(&self) -> u32 {
        self.p95_frame_time_us
    }
}
