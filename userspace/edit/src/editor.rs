//! Ind OS Text Editor State Machine
//!
//! Standards:
//! - INV-DEVX-001: Bounded text buffer safety

use crate::buffer::TextBuffer;
use crate::types::{EditError, EditorMode};

pub struct EditorSession {
    pub buffer: TextBuffer,
    pub mode: EditorMode,
    pub filename: &'static str,
    pub dirty: bool,
}

impl EditorSession {
    pub const fn new(filename: &'static str) -> Self {
        Self {
            buffer: TextBuffer::new(),
            mode: EditorMode::Normal,
            filename,
            dirty: false,
        }
    }

    pub fn handle_key(&mut self, key: u8) -> Result<(), EditError> {
        match self.mode {
            EditorMode::Normal => match key {
                b'i' => {
                    self.mode = EditorMode::Insert;
                }
                b'h' => {
                    if self.buffer.cursor_col > 0 {
                        self.buffer.cursor_col -= 1;
                    }
                }
                b'l' => {
                    let len = self.buffer.line_lens[self.buffer.cursor_row];
                    if self.buffer.cursor_col < len {
                        self.buffer.cursor_col += 1;
                    }
                }
                b'j' => {
                    if self.buffer.cursor_row + 1 < self.buffer.line_count {
                        self.buffer.cursor_row += 1;
                    }
                }
                b'k' => {
                    if self.buffer.cursor_row > 0 {
                        self.buffer.cursor_row -= 1;
                    }
                }
                _ => {}
            },
            EditorMode::Insert => match key {
                0x1b => {
                    // Escape key -> Normal mode
                    self.mode = EditorMode::Normal;
                }
                b'\n' | b'\r' => {
                    self.buffer.insert_newline()?;
                    self.dirty = true;
                }
                0x08 | 0x7f => {
                    // Backspace
                    self.buffer.delete_char()?;
                    self.dirty = true;
                }
                c => {
                    self.buffer.insert_char(c)?;
                    self.dirty = true;
                }
            },
            EditorMode::Command => {
                if key == 0x1b {
                    self.mode = EditorMode::Normal;
                }
            }
        }
        Ok(())
    }
}
