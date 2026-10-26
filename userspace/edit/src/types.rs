//! Ind OS Text Editor Types
//!
//! Standards:
//! - INV-DEVX-001: Bounded text buffer safety

pub const MAX_LINES: usize = 64;
pub const MAX_COLS: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    BufferFull,
    LineTooLong,
    OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
}
