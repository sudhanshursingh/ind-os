# Ind OS Stress, Soak & Lock Discipline Specification (SP-15 / Gate G2)

**Document ID:** `INDOS-SPEC-STRESS-01`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.8, §14, §19.2, §24.1)  
**TCB Subsystems:** `kernel/src/task/`, `kernel/src/mm/`, `kernel/src/sync/`, `kernel/src/arch/`  
**Review Status:** RATIFIED (Phase 2 Core Kernel Exit / Gate G2 Baseline)  

---

## 1. Architectural Objectives & Scope

Sprint SP-15 marks the culmination of **Phase 2 (Core Kernel)** and the entry to **Gate G2**. The objective is to subject the core kernel abstractions (virtual memory, processes, preemptive scheduler, syscall dispatcher, and object model) to sustained stress, soak testing, and lock discipline auditing:

1. **Process & Multitasking Soak:** Sustained concurrent execution of isolated user-space processes under continuous timer preemption, measuring context-switch latency and proving long-term scheduler stability.
2. **Memory Pressure Soak:** Extreme frame allocation and deallocation under memory pressure, verifying bounds enforcement, graceful `OutOfMemory` handling, and absence of physical frame leakage.
3. **Address-Space Isolation Verification:** Empirical proof of address-space separation (cross-boundary or unauthorized writes trigger synchronous translation/permission faults trapped and contained by EL1).
4. **Lock Discipline & Deadlock Freedom:** Strict total ordering of lock acquisition across all kernel singletons (`CONSOLE`, `FRAME_ALLOCATOR`, `LOCKED_HEAP`, `SCHEDULER`, `ENDPOINT_REGISTRY`, `OBJECT_TABLE`), accompanied by mandatory `lock_irqsave` discipline during exception/interrupt contexts.
5. **Unsafe Code Budget Audit:** 100% line-by-line audit of all freestanding `unsafe` blocks in the TCB against Bible Section 24.1.

---

## 2. Machine Invariants (Phase 2 Exit / Gate G2)

1. **`INV-STRESS-001` (Preemptive Scheduling Stability under Soak):**
   The kernel scheduler must sustain at least 50 context switches and 40 preemptions between concurrent processes without scheduling starvation, priority inversion, or deadlocks.

2. **`INV-STRESS-002` (Memory Allocation Recovery under Pressure):**
   Exhaustion of physical page frames or heap capacity must deterministically return typed error results (`Err(MemoryError::OutOfMemory)`) without panicking, and subsequent deallocations must restore 100% of available memory.

3. **`INV-STRESS-003` (Address-Space Hard Isolation):**
   Any execution stream attempting access outside its mapped translation range (`TTBR0_EL1` unmapped virtual address) or attempting kernel writes from user mode (`AP[2:1] = 0b01`) must generate an unrecoverable Synchronous Data/Instruction Abort at lower EL, caught and isolated by the kernel without compromising kernel execution integrity.

4. **`INV-STRESS-004` (Strict Lock Ordering & Deadlock Freedom):**
   Locks in IndK adhere to a strict partial ordering to prevent AB-BA deadlocks:
   $$\text{CONSOLE} < \text{FRAME\_ALLOCATOR} < \text{LOCKED\_HEAP} < \text{SCHEDULER} < \text{ENDPOINT\_REGISTRY} < \text{OBJECT\_TABLE}$$
   Locks acquired within interrupt context must exclusively use `lock_irqsave()` to prevent recursive re-entry deadlocks.

---

## 3. Lock Acquisition Hierarchy

```text
+-----------------------------------------------------------+
| Level 1: CONSOLE (PL011 UART early & thread-safe print)   |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
| Level 2: FRAME_ALLOCATOR (Physical Page Frame Allocator)  |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
| Level 3: LOCKED_HEAP (Kernel Dynamic Heap Allocator)      |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
| Level 4: SCHEDULER (Process Run Queues & State Machine)   |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
| Level 5: ENDPOINT_REGISTRY (Synchronous IPC Endpoints)    |
+-----------------------------+-----------------------------+
                              |
+-----------------------------v-----------------------------+
| Level 6: OBJECT_TABLE (Centralized Kernel Object Registry)|
+-----------------------------------------------------------+
```

* **Rule 1 (Order Preservation):** If a thread holds a lock at Level $N$, it may only acquire a lock at Level $M > N$.
* **Rule 2 (Interrupt Safety):** Any lock shared between thread context and IRQ handler context must be acquired with `lock_irqsave()`, disabling CPU IRQs in `DAIF` for the duration of the critical section.
* **Rule 3 (RAII Lifetime):** Spinlock guards must never be manually leaked; scopes must be strictly delimited.

---

## 4. Unsafe Code Budget Discipline (Bible §24.1)

All `unsafe` blocks across `kernel/src/` must satisfy:
1. **Explicit `// SAFETY:` rationale:** Documenting exact hardware prerequisites, pointer validity, and alignment.
2. **Minimal scope:** Unsafe blocks contain only the privileged instruction or raw pointer dereference; invariant verification and boundary checks occur outside in safe Rust.
3. **Zero unsafe expansions:** No new unsafe blocks permitted in capability, IPC, object table, or policy evaluation logic.
