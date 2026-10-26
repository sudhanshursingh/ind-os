//! System Call Subsystem for IndK
//! Implements AArch64 Register ABI dispatch and user pointer validation per Bible §8.8 (SP-11 / SP-12 / SP-13).

use crate::{log_info, log_warn, print};
use crate::mm::address_space::{is_valid_user_address, USER_SPACE_END};
use crate::task::{terminate_current_process, UserContext, SCHEDULER, CURRENT_PROCESS, SchedHint};

// Standard IndK System Call Numbers
pub const SYS_YIELD: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_GETPID: u64 = 3;
pub const SYS_SCHED_HINT: u64 = 4;

// Error codes
pub const EPERM: i64 = -1;
pub const ENOENT: i64 = -2;
pub const EBADF: i64 = -9;
pub const EFAULT: i64 = -14;
pub const EINVAL: i64 = -22;
pub const ENOSYS: i64 = -38;

/// Validates that a user-provided memory buffer falls strictly within user virtual memory
/// and strictly outside the NULL pointer trap zone (INV-MEM-001).
pub fn validate_user_buffer(ptr: usize, len: usize) -> Result<(), i64> {
    if len == 0 {
        return Ok(());
    }

    if !is_valid_user_address(ptr) {
        return Err(EFAULT);
    }

    let end = ptr.checked_add(len).ok_or(EFAULT)?;
    if end > USER_SPACE_END {
        return Err(EFAULT);
    }

    Ok(())
}

/// Dispatches system calls issued from EL0 via SVC #0
/// Register ABI:
///   x8: syscall number
///   x0..x5: arguments
///   x0: return value
pub fn handle_syscall(context: &mut UserContext) {
    let syscall_num = context.x[8];
    let arg0 = context.x[0];
    let arg1 = context.x[1];
    let arg2 = context.x[2];

    let ret: i64 = match syscall_num {
        SYS_YIELD => {
            // Voluntary yield to next scheduled process
            SCHEDULER.lock().yield_current(context);
            0
        }

        SYS_WRITE => {
            let fd = arg0 as usize;
            let buf_ptr = arg1 as usize;
            let count = arg2 as usize;

            // Only stdout (1) and stderr (2) supported in early microkernel
            if fd != 1 && fd != 2 {
                EBADF
            } else if let Err(err) = validate_user_buffer(buf_ptr, count) {
                err
            } else {
                // Print user bytes to serial console
                // SAFETY: validate_user_buffer verified that [buf_ptr..buf_ptr + count] is within valid user virtual address bounds.
                let slice = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, count) };
                for &b in slice {
                    print!("{}", b as char);
                }
                count as i64
            }
        }

        SYS_EXIT => {
            let status = arg0 as i32;
            log_info!("proc", "User process issued sys_exit(status: {})", status);
            let has_more = SCHEDULER.lock().exit_current(status, context);
            if !has_more {
                // Check if running in SP-11 single process mode
                terminate_current_process(status);
            }
            0
        }

        SYS_GETPID => {
            let pid = SCHEDULER.lock().current_process().map(|p| p.pid).or_else(|| {
                CURRENT_PROCESS.lock().as_ref().map(|p| p.pid)
            }).unwrap_or(0);
            pid as i64
        }

        SYS_SCHED_HINT => {
            let hint_val = arg0 as u32;
            if let Some(hint) = SchedHint::from_u32(hint_val) {
                let pid = SCHEDULER.lock().current_process().map(|p| p.pid).unwrap_or(0);
                SCHEDULER.lock().sched_hint(pid, hint);
                0
            } else {
                EINVAL
            }
        }

        _ => {
            log_warn!("syscall", "Unknown syscall number: {}", syscall_num);
            ENOSYS
        }
    };

    context.x[0] = ret as u64;
}
