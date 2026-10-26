//! Ind OS Bounded Text Buffer Implementation
//!
//! Standards:
//! - INV-DEVX-001: Bounded text buffer safety with zero heap allocations

use crate::types::{EditError, MAX_COLS, MAX_LINES};

pub struct TextBuffer {
    pub lines: [[u8; MAX_COLS]; MAX_LINES],
    pub line_lens: [usize; MAX_LINES],
    pub line_count: usize,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl TextBuffer {
    pub const fn new() -> Self {
        Self {
            lines: [[0u8; MAX_COLS]; MAX_LINES],
            line_lens: [0usize; MAX_LINES],
            line_count: 1,
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn insert_char(&mut self, c: u8) -> Result<(), EditError> {
        let r = self.cursor_row;
        let c_col = self.cursor_col;
        let cur_len = self.line_lens[r];

        if cur_len >= MAX_COLS {
            return Err(EditError::LineTooLong);
        }

        // Shift characters right
        let mut i = cur_len;
        while i > c_col {
            self.lines[r][i] = self.lines[r][i - 1];
            i -= 1;
        }

        self.lines[r][c_col] = c;
        self.line_lens[r] += 1;
        self.cursor_col += 1;

        Ok(())
    }

    pub fn delete_char(&mut self) -> Result<(), EditError> {
        let r = self.cursor_row;
        let c_col = self.cursor_col;
        if c_col == 0 {
            return Ok(());
        }

        let cur_len = self.line_lens[r];
        let mut i = c_col - 1;
        while i + 1 < cur_len {
            self.lines[r][i] = self.lines[r][i + 1];
            i += 1;
        }

        self.line_lens[r] -= 1;
        self.cursor_col -= 1;
        Ok(())
    }

    pub fn insert_newline(&mut self) -> Result<(), EditError> {
        if self.line_count >= MAX_LINES {
            return Err(EditError::BufferFull);
        }

        let r = self.cursor_row;
        let c_col = self.cursor_col;
        let cur_len = self.line_lens[r];

        // Shift lines down
        let mut line_idx = self.line_count;
        while line_idx > r + 1 {
            self.lines[line_idx] = self.lines[line_idx - 1];
            self.line_lens[line_idx] = self.line_lens[line_idx - 1];
            line_idx -= 1;
        }

        // Copy tail of current line to new line
        let split_len = cur_len - c_col;
        let mut j = 0;
        while j < split_len {
            self.lines[r + 1][j] = self.lines[r][c_col + j];
            j += 1;
        }
        self.line_lens[r + 1] = split_len;
        self.line_lens[r] = c_col;

        self.line_count += 1;
        self.cursor_row += 1;
        self.cursor_col = 0;

        Ok(())
    }

    pub fn load_str(&mut self, text: &str) -> Result<(), EditError> {
        self.line_count = 0;
        self.cursor_row = 0;
        self.cursor_col = 0;

        for line in text.lines() {
            if self.line_count >= MAX_LINES {
                return Err(EditError::BufferFull);
            }
            let b = line.as_bytes();
            let copy_len = if b.len() > MAX_COLS { MAX_COLS } else { b.len() };
            for j in 0..copy_len {
                self.lines[self.line_count][j] = b[j];
            }
            self.line_lens[self.line_count] = copy_len;
            self.line_count += 1;
        }

        if self.line_count == 0 {
            self.line_count = 1;
            self.line_lens[0] = 0;
        }

        Ok(())
    }

    pub fn line_as_str(&self, row: usize) -> &str {
        if row >= self.line_count {
            return "";
        }
        core::str::from_utf8(&self.lines[row][..self.line_lens[row]]).unwrap_or("")
    }
}
