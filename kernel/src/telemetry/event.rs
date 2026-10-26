//! Telemetry Event Schema & Types for IndK
//! Implements compact, structured kernel event schemas per INDOS-SPEC-TELEM-01 and REQ-KRN-113.

/// Discrete kernel event categories captured by the telemetry subsystem
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelemetryEventKind {
    /// Context switch event: prev_pid, next_pid, next_priority
    SchedulerSwitch {
        prev_pid: u32,
        next_pid: u32,
        priority: u8,
    },
    /// Inter-process communication message transfer
    IpcTransfer {
        sender_pid: u32,
        receiver_pid: u32,
        endpoint_id: u32,
    },
    /// Memory pressure tier transition or frame reclamation action
    MemoryPressure {
        level: u8,
        free_frames: usize,
        total_frames: usize,
    },
    /// Capability or policy access control violation
    SecurityViolation {
        source_pid: u32,
        violation_code: u32,
    },
}

impl TelemetryEventKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::SchedulerSwitch { .. } => "SchedulerSwitch",
            Self::IpcTransfer { .. } => "IpcTransfer",
            Self::MemoryPressure { .. } => "MemoryPressure",
            Self::SecurityViolation { .. } => "SecurityViolation",
        }
    }
}

/// A structured telemetry event record with monotonic sequence numbering
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TelemetryEvent {
    pub seq: u64,
    pub timestamp_ticks: u64,
    pub kind: TelemetryEventKind,
}

impl TelemetryEvent {
    pub const fn new(seq: u64, timestamp_ticks: u64, kind: TelemetryEventKind) -> Self {
        Self {
            seq,
            timestamp_ticks,
            kind,
        }
    }
}
