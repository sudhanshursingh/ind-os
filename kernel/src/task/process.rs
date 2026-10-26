//! Process Abstraction & Control Block (PCB) for IndK
//! Implements per-process execution state, register contexts, and memory space binding per Bible §8.8 (SP-11).

use crate::loader::indbin::{load_indbin, LoaderError};
use crate::mm::address_space::AddressSpace;

pub type ProcessId = u32;

/// Process lifecycle state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Exited(i32),
}

/// AArch64 User Execution Context (saved across EL0 <-> EL1 transitions)
/// Strictly 16-byte aligned, 288 bytes total size.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct UserContext {
    pub x: [u64; 31],    // x0..x30 (248 bytes)
    pub sp_el0: u64,     // User stack pointer (offset 248)
    pub elr_el1: u64,    // User program counter (offset 256)
    pub spsr_el1: u64,   // User processor state (offset 264)
    pub esr_el1: u64,    // Exception syndrome (offset 272)
    pub _pad: u64,       // 16-byte alignment pad (offset 280)
}

impl UserContext {
    pub const fn empty() -> Self {
        Self {
            x: [0; 31],
            sp_el0: 0,
            elr_el1: 0,
            spsr_el1: 0, // EL0t with all interrupts unmasked
            esr_el1: 0,
            _pad: 0,
        }
    }
}

/// AArch64 Kernel Callee-Saved Context for user execution trampolines
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default)]
pub struct KernelContext {
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64, // Frame pointer (fp)
    pub x30: u64, // Link register (lr)
    pub sp: u64,  // Kernel stack pointer
    pub _pad: [u64; 3], // Pad to multiple of 16 bytes
}

/// Process Control Block (PCB)
pub struct Process {
    pub pid: ProcessId,
    pub name: &'static str,
    pub state: ProcessState,
    pub address_space: AddressSpace,
    pub context: UserContext,
    pub exit_code: Option<i32>,
}

impl Process {
    /// Creates and initializes a new user process from an INDBIN executable image
    pub fn create_from_indbin(
        pid: ProcessId,
        name: &'static str,
        image: &[u8],
    ) -> Result<Self, LoaderError> {
        let mut address_space = AddressSpace::new(pid as u16)?;
        let (entry_point, stack_top) = load_indbin(&mut address_space, image)?;

        let mut context = UserContext::empty();
        context.elr_el1 = entry_point as u64;
        context.sp_el0 = stack_top as u64;
        context.spsr_el1 = 0x0000_0000; // EL0t execution mode

        Ok(Self {
            pid,
            name,
            state: ProcessState::Ready,
            address_space,
            context,
            exit_code: None,
        })
    }

    /// Checks if the process has completed execution
    pub fn is_exited(&self) -> bool {
        matches!(self.state, ProcessState::Exited(_))
    }
}
