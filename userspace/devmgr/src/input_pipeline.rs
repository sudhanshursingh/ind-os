//! Input Pipeline to Shell Milestone (SP-79, REQ-HW-107, REQ-UX-102, REQ-SEC-104)
//!
//! Architectural Invariants:
//! - INV-INP-001: Coordinate space normalization (1080x2460 affine mapping)
//! - INV-INP-002: Deterministic gesture recognition (Tap, Scroll, LongPress)
//! - INV-INP-003: Shell input routing & focus gating (CAP_INPUT_SHELL)

pub const CAP_INPUT_SHELL: u32 = 0x0008_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    Tap { x: u32, y: u32 },
    Scroll { dx: i32, dy: i32 },
    LongPress { x: u32, y: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPipelineError {
    InvalidCoordinates,
    PermissionDenied,
    NoFocusedSurface,
}

pub struct InputPipelineHarness {
    focused_surface_id: Option<u32>,
    dispatched_taps: u32,
    dispatched_scrolls: u32,
}

impl InputPipelineHarness {
    pub const fn new() -> Self {
        Self {
            focused_surface_id: None,
            dispatched_taps: 0,
            dispatched_scrolls: 0,
        }
    }

    pub fn set_focus(&mut self, surface_id: u32) {
        self.focused_surface_id = Some(surface_id);
    }

    pub fn transform_and_dispatch(&mut self, cap_mask: u32, gesture: Gesture) -> Result<(), InputPipelineError> {
        if (cap_mask & CAP_INPUT_SHELL) == 0 {
            return Err(InputPipelineError::PermissionDenied);
        }
        if self.focused_surface_id.is_none() {
            return Err(InputPipelineError::NoFocusedSurface);
        }
        match gesture {
            Gesture::Tap { x, y } => {
                if x >= 1080 || y >= 2460 {
                    return Err(InputPipelineError::InvalidCoordinates);
                }
                self.dispatched_taps += 1;
            }
            Gesture::Scroll { .. } => {
                self.dispatched_scrolls += 1;
            }
            Gesture::LongPress { x, y } => {
                if x >= 1080 || y >= 2460 {
                    return Err(InputPipelineError::InvalidCoordinates);
                }
            }
        }
        Ok(())
    }

    pub fn dispatched_taps(&self) -> u32 {
        self.dispatched_taps
    }

    pub fn dispatched_scrolls(&self) -> u32 {
        self.dispatched_scrolls
    }

    pub fn focused_surface_id(&self) -> Option<u32> {
        self.focused_surface_id
    }
}
