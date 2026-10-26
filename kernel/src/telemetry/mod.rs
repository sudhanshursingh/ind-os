//! Kernel Telemetry Subsystem for IndK
//! Implements read-only circular ring buffer of scheduler/IPC/memory events for later AIS consumption (REQ-KRN-113).

pub mod event;
pub mod ring_buffer;

pub use event::{TelemetryEvent, TelemetryEventKind};
pub use ring_buffer::{TelemetryRingBuffer, RING_CAPACITY};

use crate::capability::{CapRights, Capability};
use crate::sync::Spinlock;
use crate::syscall::EPERM;

/// Global Kernel Telemetry Ring Buffer Singleton
pub static TELEMETRY_BUFFER: Spinlock<TelemetryRingBuffer> =
    Spinlock::new(TelemetryRingBuffer::new());

/// Emits an event into the global telemetry buffer using current tick count
pub fn emit_event(kind: TelemetryEventKind) -> u64 {
    // In bare-metal ARM64 virt, query ticks or monotonically increment
    let ticks = 100; // Sample tick or hardware timer reference
    TELEMETRY_BUFFER.lock().emit(ticks, kind)
}

/// Capability-guarded write interface to verify user-space write denial (REQ-KRN-113)
/// Telemetry buffers are strictly read-only for userspace. Any write attempt from
/// an unprivileged caller (PID > 0) or without CapRights::WRITE is denied with EPERM.
pub fn write_telemetry_guarded(
    caller_pid: u32,
    cap: &Capability,
    _data: &[u8],
) -> Result<(), i64> {
    // 1. Unprivileged user-mode caller check (REQ-KRN-113)
    if caller_pid > 0 {
        return Err(EPERM);
    }

    // 2. Capability rights enforcement
    if !cap.rights.contains(CapRights::WRITE) {
        return Err(EPERM);
    }

    Ok(())
}
