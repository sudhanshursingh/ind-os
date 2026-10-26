//! Task and Process Management Subsystem for IndK
//! Implements Process Control Blocks, User Contexts, and Process Execution.

pub mod process;

pub use process::{KernelContext, Process, ProcessId, ProcessState, UserContext};

use crate::sync::Spinlock;

/// Global holder for the currently active user process
pub static CURRENT_PROCESS: Spinlock<Option<Process>> = Spinlock::new(None);

/// Global holder for kernel continuation context across user execution
pub static mut KERNEL_SAVED_CONTEXT: KernelContext = KernelContext {
    x19: 0,
    x20: 0,
    x21: 0,
    x22: 0,
    x23: 0,
    x24: 0,
    x25: 0,
    x26: 0,
    x27: 0,
    x28: 0,
    x29: 0,
    x30: 0,
    sp: 0,
    _pad: [0; 3],
};

extern "C" {
    fn kernel_call_user(user_ctx: *const UserContext, kctx: *mut KernelContext) -> i32;
    fn kernel_return_from_user(kctx: *const KernelContext, exit_code: i32) -> !;
}

/// Executes a user process in EL0 and blocks until it yields or exits
pub fn run_user_process(mut process: Process) -> i32 {
    process.state = ProcessState::Running;
    process.address_space.activate();

    let user_ctx_ptr = &process.context as *const UserContext;
    let kctx_ptr = core::ptr::addr_of_mut!(KERNEL_SAVED_CONTEXT);

    // Save active process in CURRENT_PROCESS
    *CURRENT_PROCESS.lock() = Some(process);

    // Call into assembly user entry trampoline (saves kernel registers into kctx, then eret to EL0)
    // SAFETY: user_ctx_ptr points to valid UserContext with EL0t configuration and valid stack/PC.
    let exit_code = unsafe {
        kernel_call_user(user_ctx_ptr, kctx_ptr)
    };

    // Retrieve finished process and record exit code
    if let Some(ref mut proc) = *CURRENT_PROCESS.lock() {
        proc.state = ProcessState::Exited(exit_code);
        proc.exit_code = Some(exit_code);
    }

    exit_code
}

/// Exits current user process and returns back to the kernel execution thread
pub fn terminate_current_process(exit_code: i32) -> ! {
    // SAFETY: KERNEL_SAVED_CONTEXT contains verified callee-saved registers and stack from kernel_call_user.
    unsafe {
        let kctx_ptr = core::ptr::addr_of!(KERNEL_SAVED_CONTEXT);
        kernel_return_from_user(kctx_ptr, exit_code);
    }
}
