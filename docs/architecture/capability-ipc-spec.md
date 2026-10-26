# Ind OS Capability & IPC Subsystem Specification (SP-13)

**Document ID:** `INDOS-SPEC-CAP-01` / `INDOS-SPEC-IPC-01`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.8, §8.9, §10)  
**TCB Subsystems:** `kernel/src/capability/`, `kernel/src/ipc/`, `kernel/src/syscall/`  
**Review Status:** RATIFIED (Phase 2 Core Kernel Sprint SP-13 Baseline)  

---

## 1. Architectural Overview & Security Objectives

In Ind OS, authority is strictly capability-based. In accordance with **Rule N-04** (Security is enforced, never requested) and **Rule N-05** (Zero ambient authority), processes possess zero ambient authority. All interactions with kernel objects, hardware devices, memory regions, and other processes must occur via explicit, typed, unforgeable capability tokens stored in the process's Capability Table (`CapTable`).

### Core Invariants

1. **`INV-CAP-001` (Monotonic Attenuation):**
   When deriving, minting, or delegating a capability, the granted rights $R_{target}$ MUST be a subset of the parent capability's rights $R_{source}$:
   $$\forall R_{target}, R_{source} : (R_{target} \ \& \ \sim R_{source}) = 0$$
   Authority may shrink or remain invariant across delegation chains, but can NEVER expand. Any attempt to expand authority fails with `EPERM`.

2. **`INV-CAP-002` (Revocation Soundness):**
   Revocation of a capability slot invalidates that capability and recursively invalidates all transitive child capabilities derived from it along the delegation tree. Once revoked, any subsequent invocation fails with `EBADF`.

3. **`INV-IPC-001` (Bounded-Latency Synchronous Rendezvous):**
   IPC endpoints provide zero-copy register-mediated synchronous message passing. Senders and receivers rendezvous atomically. Message delivery and capability delegation occur synchronously without intermediate heap buffering in the fast path.

---

## 2. Capability Model & Token Structure

### 2.1 Capability Rights (`CapRights`)

Rights are modeled as orthogonal bitflags:
* `READ` (`1 << 0`): Authority to inspect object state or receive from an IPC endpoint.
* `WRITE` (`1 << 1`): Authority to mutate object state or send to an IPC endpoint.
* `GRANT` (`1 << 2`): Authority to mint derived tokens or delegate the capability across an IPC endpoint.
* `CALL` (`1 << 3`): Authority to invoke synchronous rendezvous RPC on an IPC endpoint.

### 2.2 Kernel Object References (`CapType`)

Capabilities guard typed kernel objects:
* `Endpoint(EndpointId)`: Synchronous IPC message channel.
* `Process(ProcessId)`: Process execution control and lifecycle query.
* `MemoryRegion { base, size }`: Physical/virtual memory address range.
* `SyscallBitmap(u64)`: Process syscall authority filter.

### 2.3 Capability Token (`Capability`)

```rust
pub struct Capability {
    pub cap_type: CapType,
    pub rights: CapRights,
    pub badge: u64,
    pub is_valid: bool,
    pub parent_slot: Option<u32>, // Lineage tracking for recursive revocation
}
```

---

## 3. Synchronous IPC & Rendezvous Protocol

### 3.1 Endpoint States

Each endpoint maintains atomic rendezvous state:
* `Idle`: No thread waiting.
* `SenderWaiting { sender_pid, msg, transferred_cap }`: Sender arrived first; thread blocked awaiting receiver.
* `ReceiverWaiting { receiver_pid }`: Receiver arrived first; thread blocked awaiting sender.

### 3.2 Register-Based Message ABI

Short messages transfer directly across AArch64 registers:
* `x0`: Endpoint capability handle.
* `x1`: Message label / protocol discriminator.
* `x2..x5`: Inline data payload (up to 32 bytes).
* Capability Transfer: Sender designates a capability slot in `x4`; if possessed with `GRANT` right, it is deposited into receiver's `CapTable` with attenuated rights.

---

## 4. System Call Surface Freeze (Bible §8.8 / SP-13)

| Number | Name | Arguments | Description |
| :---: | :--- | :--- | :--- |
| `0` | `SYS_YIELD` | none | Voluntarily yields remaining time slice to next ready process. |
| `1` | `SYS_WRITE` | `x0`: fd, `x1`: ptr, `x2`: len | Writes buffer to console (requires fd 1 or 2). |
| `2` | `SYS_EXIT` | `x0`: status | Terminates calling process and notifies scheduler. |
| `3` | `SYS_GETPID` | none | Returns current Process ID. |
| `4` | `SYS_SCHED_HINT` | `x0`: hint | Advisory scheduler hint (advisory-only, Rule N-04). |
| `5` | `SYS_CAP_MINT` | `x0`: src_handle, `x1`: rights, `x2`: badge | Mints derived capability with monotonic rights attenuation (`INV-CAP-001`). |
| `6` | `SYS_CAP_REVOKE` | `x0`: handle | Revokes capability and all derived child tokens (`INV-CAP-002`). |
| `7` | `SYS_IPC_SEND` | `x0`: ep_handle, `x1`: label, `x2..x3`: data, `x4`: cap_handle | Synchronous IPC send with optional capability delegation. |
| `8` | `SYS_IPC_RECV` | `x0`: ep_handle, `x1`: out_buf_ptr | Synchronous IPC receive into user buffer. |
| `9` | `SYS_IPC_CALL` | `x0`: ep_handle, `x1`: label, `x2..x3`: data | Atomic synchronous request-reply RPC. |

---

## 5. Security & Isolation Invariants

* **Permission Bitmap Scaffolding:** Every process PCB holds a 64-bit `syscall_mask`. Attempts to issue disallowed system calls are rejected immediately with `-EPERM`.
* **Zero Ambient Authority:** System calls cannot forge object pointers or invoke endpoints by raw IDs; handles must reference valid slots in the process `CapTable`.
* **Containment:** Kernel memory allocations in capability/IPC tables are bounded and pre-allocated (`MAX_CAPABILITIES = 64`, `MAX_ENDPOINTS = 32`), guaranteeing determinism and zero dynamic memory exhaustion during IPC rendezvous.
