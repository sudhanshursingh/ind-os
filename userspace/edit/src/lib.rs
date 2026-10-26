//! Ind OS Modal Text Editor Library (`indus-edit`)
//!
//! Standards:
//! - INV-DEVX-001: Bounded text buffer safety
//! - INV-DEVX-002: Capability-mediated remote stream

#![no_std]

pub mod buffer;
pub mod editor;
pub mod types;

pub use buffer::TextBuffer;
pub use editor::EditorSession;
pub use types::{EditError, EditorMode, MAX_COLS, MAX_LINES};
