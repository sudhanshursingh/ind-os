//! System Call Subsystem for IndK
//! Implements AArch64 Register ABI dispatch, per-process permission bitmaps,
//! capabilities, and synchronous IPC endpoints per Bible §8.8 / §10 (SP-11 / SP-12 / SP-13).

use crate::capability::{CapRights, CapType};
use crate::ipc::{IpcMessage, IpcRecvResult, IpcSendResult, ENDPOINT_REGISTRY};
use crate::mm::address_space::{is_valid_user_address, USER_SPACE_END};
use crate::task::{terminate_current_process, SchedHint, UserContext, CURRENT_PROCESS, SCHEDULER};
use crate::{log_info, log_warn, print};

// Standard IndK System Call Numbers
pub const SYS_YIELD: u64 = 0;
pub const SYS_WRITE: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_GETPID: u64 = 3;
pub const SYS_SCHED_HINT: u64 = 4;
pub const SYS_CAP_MINT: u64 = 5;
pub const SYS_CAP_REVOKE: u64 = 6;
pub const SYS_IPC_SEND: u64 = 7;
pub const SYS_IPC_RECV: u64 = 8;
pub const SYS_IPC_CALL: u64 = 9;

// Error codes
pub const EPERM: i64 = -1;
pub const ENOENT: i64 = -2;
pub const EBADF: i64 = -9;
pub const ENOMEM: i64 = -12;
pub const EFAULT: i64 = -14;
pub const EBUSY: i64 = -16;
pub const EEXIST: i64 = -17;
pub const EINVAL: i64 = -22;
pub const ENOSYS: i64 = -38;
pub const ETIMEDOUT: i64 = -110;

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
    let arg3 = context.x[3];
    let arg4 = context.x[4];

    // Check per-process syscall permission bitmap (Bible §8.8 / SP-13 pre-capability scaffolding)
    let (has_perm, pid) = {
        let sched = SCHEDULER.lock();
        if let Some(proc) = sched.current_process() {
            let allowed = if syscall_num < 64 {
                (proc.syscall_mask & (1 << syscall_num)) != 0
            } else {
                false
            };
            (allowed, proc.pid)
        } else {
            (true, 0)
        }
    };

    if !has_perm {
        log_warn!("syscall", "Process {} denied syscall {} by permission bitmap (EPERM)", pid, syscall_num);
        context.x[0] = EPERM as u64;
        return;
    }

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

        SYS_CAP_MINT => {
            // arg0: src_handle, arg1: new_rights_bits, arg2: badge
            let src_handle = arg0 as u32;
            let new_rights = CapRights::from_bits(arg1 as u32);
            let badge = arg2;

            let mut sched = SCHEDULER.lock();
            if let Some(proc) = sched.current_process_mut() {
                match proc.cap_table.mint(src_handle, new_rights, badge) {
                    Ok(new_handle) => new_handle as i64,
                    Err(err) => err,
                }
            } else {
                EPERM
            }
        }

        SYS_CAP_REVOKE => {
            // arg0: handle to revoke
            let handle = arg0 as u32;
            let mut sched = SCHEDULER.lock();
            if let Some(proc) = sched.current_process_mut() {
                match proc.cap_table.revoke(handle) {
                    Ok(revoked_count) => revoked_count as i64,
                    Err(err) => err,
                }
            } else {
                EPERM
            }
        }

        SYS_IPC_SEND => {
            // arg0: ep_handle, arg1: label, arg2: data0, arg3: data1, arg4: cap_transfer_handle
            let ep_handle = arg0 as u32;
            let label = arg1;
            let data = [arg2, arg3, 0, 0];
            let transfer_handle = if arg4 != 0 && arg4 != 0xFFFF_FFFF {
                Some(arg4 as u32)
            } else {
                None
            };

            let mut sched = SCHEDULER.lock();
            if let Some(proc) = sched.current_process_mut() {
                let sender_pid = proc.pid;
                let ep_id = match proc.cap_table.get(ep_handle) {
                    Ok(cap) => {
                        if !cap.rights.contains(CapRights::WRITE) && !cap.rights.contains(CapRights::CALL) {
                            return context.x[0] = EPERM as u64;
                        }
                        match cap.cap_type {
                            CapType::Endpoint(id) => id,
                            _ => return context.x[0] = EINVAL as u64,
                        }
                    }
                    Err(err) => return context.x[0] = err as u64,
                };

                // Check transfer capability if requested
                let transferred_cap = if let Some(th) = transfer_handle {
                    match proc.cap_table.get(th) {
                        Ok(c) => {
                            if !c.rights.contains(CapRights::GRANT) {
                                return context.x[0] = EPERM as u64;
                            }
                            Some(*c)
                        }
                        Err(err) => return context.x[0] = err as u64,
                    }
                } else {
                    None
                };

                let msg = IpcMessage::new(label, data, transfer_handle);
                match ENDPOINT_REGISTRY.lock().send(sender_pid, ep_id, msg, transferred_cap) {
                    Ok(IpcSendResult::RendezvousDelivered { .. }) => 0,
                    Ok(IpcSendResult::SenderQueued) => {
                        // Sender blocks on synchronous rendezvous
                        proc.state = crate::task::ProcessState::Blocked;
                        SCHEDULER.lock().yield_current(context);
                        0
                    }
                    Err(err) => err,
                }
            } else {
                EPERM
            }
        }

        SYS_IPC_RECV => {
            // arg0: ep_handle, arg1: out_buf_ptr
            let ep_handle = arg0 as u32;
            let out_buf = arg1 as usize;

            let mut sched = SCHEDULER.lock();
            if let Some(proc) = sched.current_process_mut() {
                let receiver_pid = proc.pid;
                let ep_id = match proc.cap_table.get(ep_handle) {
                    Ok(cap) => {
                        if !cap.rights.contains(CapRights::READ) && !cap.rights.contains(CapRights::CALL) {
                            return context.x[0] = EPERM as u64;
                        }
                        match cap.cap_type {
                            CapType::Endpoint(id) => id,
                            _ => return context.x[0] = EINVAL as u64,
                        }
                    }
                    Err(err) => return context.x[0] = err as u64,
                };

                if out_buf != 0 {
                    if let Err(err) = validate_user_buffer(out_buf, 40) {
                        return context.x[0] = err as u64;
                    }
                }

                match ENDPOINT_REGISTRY.lock().recv(receiver_pid, ep_id) {
                    Ok(IpcRecvResult::RendezvousReceived { msg, transferred_cap, .. }) => {
                        if let Some(cap) = transferred_cap {
                            let _ = proc.cap_table.insert(cap);
                        }
                        if out_buf != 0 {
                            let slice = unsafe { core::slice::from_raw_parts_mut(out_buf as *mut u64, 5) };
                            slice[0] = msg.label;
                            slice[1] = msg.data[0];
                            slice[2] = msg.data[1];
                            slice[3] = msg.data[2];
                            slice[4] = msg.data[3];
                        }
                        0
                    }
                    Ok(IpcRecvResult::ReceiverQueued) => {
                        proc.state = crate::task::ProcessState::Blocked;
                        SCHEDULER.lock().yield_current(context);
                        0
                    }
                    Err(err) => err,
                }
            } else {
                EPERM
            }
        }

        SYS_IPC_CALL => {
            // Synchronous RPC call
            0
        }

        _ => {
            log_warn!("syscall", "Unknown syscall number: {}", syscall_num);
            ENOSYS
        }
    };

    context.x[0] = ret as u64;
}
