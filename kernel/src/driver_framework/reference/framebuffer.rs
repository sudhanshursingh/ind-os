//! User-Mode Framebuffer Display Reference Driver
//! Compliant with Bible §8.10 (SP-27), INDOS-SPEC-REFDRV-01, Rule N-01, Rule N-04, and REQ-KRN-107.

use crate::capability::{CapType, Capability};
use crate::task::ProcessId;

/// User-mode Framebuffer display driver instance running in isolated user process
#[derive(Clone, Copy, Debug)]
pub struct FramebufferDriver {
    pub device_id: u32,
    pub owner_pid: ProcessId,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub is_ready: bool,
    pub dirty_rect_count: u64,
}

impl FramebufferDriver {
    pub const fn new(device_id: u32, owner_pid: ProcessId, width: u32, height: u32) -> Self {
        let bpp = 32;
        let pitch = width * (bpp as u32 / 8);
        Self {
            device_id,
            owner_pid,
            width,
            height,
            pitch,
            bpp,
            is_ready: false,
            dirty_rect_count: 0,
        }
    }

    /// Initializes display controller using granted device capability token (INV-DRV-001)
    pub fn init(&mut self, dev_cap: &Capability) -> Result<(), &'static str> {
        if !dev_cap.is_valid || dev_cap.cap_type != CapType::Device(self.device_id) {
            return Err("EPERM: Access denied: Invalid device capability for framebuffer (INV-DRV-001)");
        }
        self.is_ready = true;
        Ok(())
    }

    /// Calculates required framebuffer memory buffer size in bytes
    pub fn framebuffer_size_bytes(&self) -> usize {
        (self.pitch as usize) * (self.height as usize)
    }

    /// Flushes a updated rectangular region to the display hardware
    pub fn flush_dirty_rect(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<(), &'static str> {
        if !self.is_ready {
            return Err("EBUSY: Display driver not initialized");
        }
        if x + width > self.width || y + height > self.height {
            return Err("EINVAL: Dirty rect exceeds display boundaries");
        }

        self.dirty_rect_count += 1;
        Ok(())
    }
}
