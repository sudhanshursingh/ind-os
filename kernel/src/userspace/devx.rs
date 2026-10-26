//! Kernel Developer Experience Verification Harness (SP-38)
//!
//! Architectural Invariants:
//! - INV-DEVX-001: Bounded text buffer safety
//! - INV-DEVX-002: Capability-mediated remote stream

pub const KERNEL_MAX_EDIT_LINES: usize = 16;
pub const KERNEL_MAX_EDIT_COLS: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelEditError {
    BufferFull,
    LineTooLong,
}

pub struct KernelTextBuffer {
    lines: [[u8; KERNEL_MAX_EDIT_COLS]; KERNEL_MAX_EDIT_LINES],
    line_lens: [usize; KERNEL_MAX_EDIT_LINES],
    line_count: usize,
}

impl KernelTextBuffer {
    pub const fn new() -> Self {
        Self {
            lines: [[0u8; KERNEL_MAX_EDIT_COLS]; KERNEL_MAX_EDIT_LINES],
            line_lens: [0usize; KERNEL_MAX_EDIT_LINES],
            line_count: 1,
        }
    }

    pub fn insert_line(&mut self, text: &str) -> Result<(), KernelEditError> {
        if self.line_count >= KERNEL_MAX_EDIT_LINES {
            return Err(KernelEditError::BufferFull);
        }
        let b = text.as_bytes();
        let len = if b.len() > KERNEL_MAX_EDIT_COLS { KERNEL_MAX_EDIT_COLS } else { b.len() };
        for i in 0..len {
            self.lines[self.line_count][i] = b[i];
        }
        self.line_lens[self.line_count] = len;
        self.line_count += 1;
        Ok(())
    }

    pub fn line_as_str(&self, idx: usize) -> &str {
        if idx >= self.line_count {
            return "";
        }
        core::str::from_utf8(&self.lines[idx][..self.line_lens[idx]]).unwrap_or("")
    }

    pub fn count(&self) -> usize {
        self.line_count
    }
}

pub struct KernelRemoteShellBridge {
    pub authenticated_cap: u32,
    pub session_active: bool,
}

impl KernelRemoteShellBridge {
    pub const fn new(authenticated_cap: u32) -> Self {
        Self {
            authenticated_cap,
            session_active: false,
        }
    }

    pub fn connect(&mut self, token: u32) -> bool {
        if token == self.authenticated_cap {
            self.session_active = true;
            true
        } else {
            false
        }
    }
}
