# Ind OS IPC Core Specification (SP-16)

**Document ID:** `INDOS-SPEC-IPC-02`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.9, §10, §14), `docs/architecture/phase-3-security-spine-roadmap.md`  
**TCB Subsystems:** `kernel/src/ipc/`, `kernel/src/capability/`, `kernel/src/syscall/`  
**Review Status:** RATIFIED (Phase 3 Security Spine Baseline)  

---

## 1. Architectural Overview & Objectives

In Ind OS, Inter-Process Communication (IPC) is the primary medium through which isolated processes interact with servers, drivers, and the kernel. In strict adherence to:
* **Rule N-04 (Security is enforced, never requested):** All IPC operations require presentation of valid, capability-guarded handles with appropriate rights (`CALL`, `READ`, `WRITE`, `GRANT`).
* **Rule N-05 (Zero ambient authority):** Processes cannot broadcast or address other processes directly by PID; all communication routes through typed endpoints or notification objects.

Sprint SP-16 expands the basic synchronous IPC mechanism from Phase 2 into a complete, high-performance IPC core:
1. **Synchronous Call/Send/Recv Endpoints:** Zero-copy rendezvous message passing in CPU registers (`x0..x7`) supporting atomic request-reply RPC (`SYS_IPC_CALL`).
2. **Capability Rights & Token Transfer:** Delegation of capabilities during IPC rendezvous, with monotonic rights attenuation and badge verification.
3. **Deadlines & Timed Rendezvous:** Timeout specifications (`deadline: u64`) supporting immediate non-blocking poll (`POLL = 0`), timed wait, and indefinite blocking (`BLOCK = u64::MAX`), with deterministic `ETIMEDOUT` returns.
4. **Asynchronous Notifications (`NotificationObject`):** Low-overhead, atomic 64-bit bitmask signaling for hardware IRQ forwarding and asynchronous event dispatch without thread blocking.
5. **Inline Capability Validation Caching:** Fast-path validation cache in per-process namespaces to keep median round-trip latency strictly under **$< 10.0\ \mu\text{s}$**.

---

## 2. Machine Invariants

1. **`INV-IPC-001` (Zero-Copy Synchronous Rendezvous):**
   Sender and receiver threads rendezvous at an endpoint; payload is transferred directly through architectural registers without intermediate kernel heap allocations or message copies.

2. **`INV-IPC-002` (Deterministic Round-Trip Latency):**
   Uncontended microkernel IPC round-trip latency (Send $\rightarrow$ Switch $\rightarrow$ Recv $\rightarrow$ Reply) must not exceed **$10.0\ \mu\text{s}$**, measured by the automated latency benchmark harness.

3. **`INV-IPC-003` (Atomic Notification Bitmask Transfer):**
   Signaling a notification atomically performs a bitwise-OR into the 64-bit notification bitmask. Waiting or polling atomically retrieves and clears the masked bits.

4. **`INV-IPC-004` (Deadline & Timeout Monotonicity):**
   Timed IPC operations measure deadlines against the monotonic hardware timer counter (`CNTPCT_EL0`). If a rendezvous cannot be completed prior to the deadline, the operation aborts cleanly and returns `Err(ETIMEDOUT)` without corrupting endpoint state.

---

## 3. Data Structures & ABI

### 3.1 Synchronous Message Format (`IpcMessage`)

```text
 63                                                              0
+---------------------------------------------------------------+
| label (64 bits: protocol opcode / message tag)                |
+---------------------------------------------------------------+
| length (64 bits: payload byte count, max 32 bytes)            |
+---------------------------------------------------------------+
| data[0] (64 bits register inline payload)                     |
+---------------------------------------------------------------+
| data[1] (64 bits register inline payload)                     |
+---------------------------------------------------------------+
| data[2] (64 bits register inline payload)                     |
+---------------------------------------------------------------+
| data[3] (64 bits register inline payload)                     |
+---------------------------------------------------------------+
| cap_transfer: Option<CapHandle> (transferred capability token)|
+---------------------------------------------------------------+
```

### 3.2 Asynchronous Notification Object (`NotificationObject`)

```text
+---------------------------------------------------------------+
| id: NotificationId (u32)                                      |
+---------------------------------------------------------------+
| state: NotificationState                                      |
|   - Idle                                                      |
|   - Waiting { receiver_pid: ProcessId }                       |
|   - Signaled { bitmask: u64 }                                 |
+---------------------------------------------------------------+
| is_allocated: bool                                            |
+---------------------------------------------------------------+
```

* `signal(notification_id, bits: u64) -> Result<(), i64>`: Atomically sets `bitmask |= bits`. If a receiver is waiting, wakes the receiver.
* `wait(notification_id, clear_mask: u64, timeout: u64) -> Result<u64, i64>`: Awaits signaled bits, returning and clearing matching bits.
* `poll(notification_id) -> Result<u64, i64>`: Non-blocking atomic read and clear of current pending bitmask.

### 3.3 Fast-Path Inline Capability Cache (`CapFastCache`)

To eliminate hash table or linear scan overhead in the hot IPC path:
```rust
pub struct CapFastCache {
    pub cached_handle: u16,
    pub cached_rights: CapRights,
    pub cached_badge: u64,
    pub generation: u16,
}
```

---

## 4. Benchmark Harness Specification

The kernel benchmark harness executes $N = 1,000$ consecutive zero-copy round-trip transfers across isolated processes, recording:
* Minimum round-trip latency ($\mu\text{s}$)
* Median / p50 round-trip latency ($\mu\text{s}$)
* p99 round-trip latency ($\mu\text{s}$)
* Maximum round-trip latency ($\mu\text{s}$)

The results are permanently committed to `docs/evidence/ipc-latency-baseline.md` as the denominator for Gate G4 performance budgets.
