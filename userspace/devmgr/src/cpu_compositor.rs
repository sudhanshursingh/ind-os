//! CPU Software Rendering Compositor (SP-77, PEN-004, REQ-UX-101)
//!
//! Architectural Invariants:
//! - INV-COMP-001: Double-buffered frame presentation architecture
//! - INV-COMP-002: Dirty rectangle bounding and validation
//! - INV-COMP-003: Capability-gated surface presentation (CAP_SURFACE_COMPOSITOR)

pub const CAP_SURFACE_COMPOSITOR: u32 = 0x0004_0000;
pub const MAX_PANEL_WIDTH: u32 = 1080;
pub const MAX_PANEL_HEIGHT: u32 = 2460;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorError {
    InvalidBounds,
    PermissionDenied,
    BufferSwapFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl DirtyRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0 &&
        self.x.saturating_add(self.width) <= MAX_PANEL_WIDTH &&
        self.y.saturating_add(self.height) <= MAX_PANEL_HEIGHT
    }
}

pub struct CpuCompositorHarness {
    front_buffer_id: u8,
    back_buffer_id: u8,
    frame_flips: u32,
    total_dirty_rects: u32,
}

impl CpuCompositorHarness {
    pub const fn new() -> Self {
        Self {
            front_buffer_id: 0,
            back_buffer_id: 1,
            frame_flips: 0,
            total_dirty_rects: 0,
        }
    }

    pub fn blit_dirty_rect(&mut self, rect: DirtyRect) -> Result<(), CompositorError> {
        if !rect.is_valid() {
            return Err(CompositorError::InvalidBounds);
        }
        self.total_dirty_rects += 1;
        Ok(())
    }

    pub fn present_frame(&mut self, cap_mask: u32) -> Result<(), CompositorError> {
        if (cap_mask & CAP_SURFACE_COMPOSITOR) == 0 {
            return Err(CompositorError::PermissionDenied);
        }
        let temp = self.front_buffer_id;
        self.front_buffer_id = self.back_buffer_id;
        self.back_buffer_id = temp;
        self.frame_flips += 1;
        Ok(())
    }

    pub fn front_buffer_id(&self) -> u8 {
        self.front_buffer_id
    }

    pub fn frame_flips(&self) -> u32 {
        self.frame_flips
    }

    pub fn total_dirty_rects(&self) -> u32 {
        self.total_dirty_rects
    }
}
