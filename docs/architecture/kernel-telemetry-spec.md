# IndOS Kernel Telemetry API v1 Architecture Specification

**Document Identifier:** `INDOS-SPEC-TELEM-01`  
**Applicable Sprints:** Sprint SP-30 (Phase 4), Bible §8.10, REQ-KRN-113  
**Status:** Approved & Formally Specified  
**Classification:** Core Kernel Architecture & Observability Spine  
**Dependencies:** `INDOS-SPEC-SCHED-01`, `INDOS-SPEC-IPC-01`, `INDOS-SPEC-OOM-01`

---

## 1. Executive Summary & Rationale

Per Bible §8.10 and REQ-KRN-113, IndOS implements an in-kernel, low-overhead observability pipeline: **Kernel Telemetry API v1**. This subsystem captures high-frequency microkernel execution events (scheduler task switches, IPC message exchanges, memory pressure transitions, and security policy violations) into a circular ring buffer in kernel memory.

This telemetry buffer provides the ground-truth observational foundation for the userspace AI Subsystem (`userspace/ais/` in SP-47), enabling real-time performance optimization, hardware profile adaptation (`REQ-AI-112`), and power/scheduling hints without modifying kernel internals or violating security boundaries.

### Core Non-Negotiables:
1. **Zero Ambient Authority & Read-Only Userspace Access (`REQ-KRN-113`):**
   The telemetry ring buffer is exposed to userspace strictly as a read-only memory region or via capability handles restricted to `CapRights::READ`. Any attempt by a user-mode caller or unprivileged daemon to write to the telemetry buffer must be rejected deterministically with `EPERM` or a hardware page translation write fault.
2. **Bounded Non-Blocking Kernel Event Emission:**
   Kernel modules emitting telemetry events must never block, allocate dynamic heap memory on the fast-path, or incur unbounded lock contention.
3. **Monotonic Sequence Numbering & Loss Detection:**
   Every telemetry record carries a monotonically increasing 64-bit sequence identifier (`seq`), allowing consumers to detect buffer overruns and event drops deterministically.

---

## 2. Telemetry Event Schema

Telemetry records are compact, fixed-size structures designed for cacheline efficiency and zero-copy consumer reads:

### 2.1 Event Kinds
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelemetryEventKind {
    /// Context switch event: prev_pid, next_pid, next_priority
    SchedulerSwitch { prev_pid: u32, next_pid: u32, priority: u8 },
    /// Synchronous or asynchronous IPC message transfer
    IpcTransfer { sender_pid: u32, receiver_pid: u32, endpoint_id: u32 },
    /// Memory pressure tier transition or reclamation action
    MemoryPressure { level: u8, free_frames: usize, total_frames: usize },
    /// Capability or policy engine access violation
    SecurityViolation { source_pid: u32, violation_code: u32 },
}
```

### 2.2 Event Record Structure
```rust
#[derive(Clone, Copy, Debug)]
pub struct TelemetryEvent {
    pub seq: u64,
    pub timestamp_ticks: u64,
    pub kind: TelemetryEventKind,
}
```

---

## 3. Circular Ring Buffer Architecture

The telemetry ring buffer maintains a fixed capacity of $N = 64$ records (`RING_CAPACITY`).

- **Ring Buffer Head & Sequence:**
  The `head_seq` is a monotonically incrementing 64-bit counter tracking the next sequence number to be assigned to an incoming event.
- **Overrun Discipline:**
  When event production exceeds ring capacity, older events are overwritten (`ring[seq % RING_CAPACITY]`). Readers querying an outdated sequence number are notified of discarded events by comparing `start_seq` against `head_seq - RING_CAPACITY`.
- **Concurrency & Synchronization:**
  Emission and consumption are synchronized via `Spinlock<TelemetryRingBuffer>`, guaranteeing atomicity across multi-core CPUs.

---

## 4. Userspace Access Control & Write-Denial Enforcement (`REQ-KRN-113`)

To guarantee system integrity, userspace daemons (including the AIS hardware profiler and AI feedback loop) can only read from the telemetry pipeline:
1. **Capability Rights Attenuation:**
   Capabilities granting access to telemetry endpoints or pages must have `CapRights::WRITE` strictly stripped (`rights = CapRights::READ`).
2. **Write-Attempt Denial Test:**
   Any system call or driver operation attempting to write, tamper with, or clear the telemetry buffer from userspace without kernel privilege is rejected with `EPERM` (`INV-SEC-001`, `REQ-KRN-113`).

---

## 5. Implementation Deliverables

1. `docs/architecture/kernel-telemetry-spec.md` — This architectural specification.
2. `kernel/src/telemetry/event.rs` — Telemetry event types and schemas.
3. `kernel/src/telemetry/ring_buffer.rs` — Bounded circular ring buffer with sequence tracking.
4. `kernel/src/telemetry/mod.rs` — Re-exports, global singleton `TELEMETRY_BUFFER`, and access enforcement.
5. `kernel/src/main.rs` — Boot verification stages 95–99.
6. `ci/verify_sp30.py` — Automated verification test harness.
