# IndOS Symmetric Multi-Processing (SMP) Architecture Specification
**Document ID:** `INDOS-SPEC-SMP-01`  
**Sprint:** SP-28 (Phase 4)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.10), Rules N-01, N-04, N-10  
**Status:** RATIFIED  
**Requirements Satisfied:** `REQ-KRN-109` (Symmetric Multi-Processing: Multi-Core Bring-up, Per-CPU Run Queues, Affinity, and IPIs)  
**Target Invariants:** `INV-SMP-001`, `INV-SMP-002`, `INV-SMP-003`  

---

## 1. Architectural Motivation & Scale

In Phase 1 and 2, IndK booted and scheduled tasks on the primary bootstrap core (CPU 0). To satisfy Bible §8.10 and `REQ-KRN-109`, Sprint SP-28 introduces **Symmetric Multi-Processing (SMP)** across up to 4 cores (`MAX_CPUS = 4`) on ARM64 QEMU virt and hardware platforms:

1. **Deterministic Multi-Core Bring-up (`INV-SMP-001`):** The primary core (CPU 0) initializes the system, discovers CPU topology, and coordinates the sequential bring-up of secondary cores (Cores 1, 2, 3) using PSCI (`CPU_ON`) conventions. Each core is assigned an isolated execution stack and core identifier (`CpuId`).
2. **Decentralized Per-CPU Run Queues (`INV-SMP-002`):** Centralized global run queues suffer severe lock contention and cache-line thrashing under multi-core scaling. Ind OS provides decentralized, per-CPU multi-level priority run queues. Each core schedules from its local run queue without taking global locks on the critical dispatch fast path.
3. **Hard Affinity Constraints (`REQ-KRN-109`):** Processes specify a bitmask affinity (`AffinityMask`). The scheduler guarantees that a task is only dispatched to cores permitted by its mask. Work-stealing and load balancing strictly preserve affinity constraints.
4. **Inter-Processor Interrupts (IPIs) (`INV-SMP-003`):** Cores coordinate cross-CPU events via ARM GIC Software Generated Interrupts (SGIs, IRQs 0–15):
   - `IpiKind::Reschedule`: Wakes up or preempts a remote core when a high-priority task arrives.
   - `IpiKind::TlbFlush`: Coordinates cross-core TLB invalidations upon address space revocation.
   - `IpiKind::FunctionCall`: Synchronously invokes remote kernel callbacks.
   - `IpiKind::Halt`: Broadcasts emergency CPU parking on kernel panic.

---

## 2. Core Subsystems

```
+--------------------------------------------------------------------------+
|                              IndOS SMP Core                              |
|                                                                          |
|  +--------------------+  +--------------------+  +--------------------+  |
|  |     CpuManager     |  |   PerCpuQueues     |  |     IpiBroker      |  |
|  | - Topology Table   |  | - Core 0 RunQueue  |  | - GIC SGI Dispatch |  |
|  | - Core 0 (Primary) |  | - Core 1 RunQueue  |  | - Reschedule IPI   |  |
|  | - Core 1..3 Bringup|  | - Core 2 RunQueue  |  | - TLB Flush IPI    |  |
|  | - PSCI Handshake   |  | - Core 3 RunQueue  |  | - Function Call    |  |
|  +---------+----------+  +---------+----------+  +---------+----------+  |
+------------|-----------------------|-----------------------|-------------+
             |                       |                       |
             v                       v                       v
+-----------------------+-----------------------+-----------------------+
|  CPU 0 (Bootstrap)    |   CPU 1 (Secondary)   |  CPU 2..3 (Secondary) |
|  - Local Scheduler    |   - Local Scheduler   |  - Local Scheduler    |
|  - SGI 0..15 Handler  |   - SGI 0..15 Handler |  - SGI 0..15 Handler  |
+-----------------------+-----------------------+-----------------------+
```

### 2.1 CPU Topology & Bring-Up (`CpuManager`)
- **Topology Descriptor:** Each core tracks `cpu_id: u8`, `mpidr: u64` (Aff0, Aff1), `state: CpuState` (`Offline`, `Starting`, `Online`, `Halted`), and `ticks: u64`.
- **Bring-up Protocol (`INV-SMP-001`):** Primary CPU 0 sets secondary core state to `Starting` and signals startup. The secondary core completes architectural setup (CPACR_EL1, SPSR, VBAR_EL1, stack) and marks its state `Online`.

### 2.2 Per-CPU Run Queues (`PerCpuRunQueue`)
- Separate 5-level priority run queues for each CPU: `Realtime` (4), `Interactive` (3), `Normal` (2), `BatchIntelligence` (1), `Idle` (0).
- Lock refinement: Each core acquires only its local queue spinlock during scheduling, reducing lock contention by $\approx 75\%$ on 4 cores.

### 2.3 Affinity Masking & Load Balancing
- `AffinityMask(u8)`: Bitmask where bit $i$ allows execution on CPU $i$.
- Dispatch invariant: If `(task.affinity.0 & (1 << cpu_id)) == 0`, CPU $cpu\_id$ shall never execute `task`.

### 2.4 Inter-Processor Interrupts (`IpiBroker`)
- Traps and routes GIC SGIs between cores.
- Maintains per-target mailboxes recording delivered and handled IPI counts.

---

## 3. Formal Invariants Matrix

| Invariant | Target Subsystem | Statement | Verification |
| :--- | :--- | :--- | :--- |
| **`INV-SMP-001`** | Core Bring-Up | Primary core brings up secondary cores into verified Online state with isolated stacks. | QEMU Stage 85; 4 cores verified Online |
| **`INV-SMP-002`** | Per-CPU Queues | Each CPU operates an isolated run queue; scheduling does not take global locks on fast path. | QEMU Stage 86; Per-CPU queue operations verified |
| **`INV-SMP-003`** | IPI & Affinity | IPI messages trigger remote core dispatch; tasks execute strictly within declared affinity mask. | QEMU Stages 87–88; Affinity constraint & IPI dispatch |

---

## 4. Definition of Done (DoD)

1. Pure safe Rust implementation in `kernel/src/smp/` with 0 `unsafe` blocks.
2. Complete implementations of `CpuManager`, `PerCpuRunQueue`, `AffinityMask`, and `IpiBroker`.
3. Headless QEMU execution covering Stages 84–89 in `kernel/src/main.rs`.
4. Verification harness `ci/verify_sp28.py` passing 6/6 checks.
5. Golden boot log and validation report recorded.
