#![no_std]

//! Ind OS UX Shell & App Framework Core Library (Phase 8, SP-60..SP-69)
//!
//! Architectural Invariants:
//! - INV-COMP-001..004: Capability display gating, zero-heap damage tracking, anti-overlay consent protection
//! - Rule N-01: Pure in-tree Rust, zero external crates in TCB
//! - Rule N-04: Zero unsafe code in new modules

pub mod compositor;
pub use compositor::{
    CompositorEngine, CompositorError, DamageRegion, InputEvent, InputEventType, Rect,
    WindowSurface, CAP_DISPLAY_OVERLAY, CAP_DISPLAY_RENDER, MAX_DIRTY_RECTS, MAX_WINDOWS,
    SYSTEM_OVERLAY_Z_BASE,
};
