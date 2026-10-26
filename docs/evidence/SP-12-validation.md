# Ind OS Sprint SP-12 Validation Report
**Sprint ID:** SP-12  
**Phase:** Phase 2 — Core Kernel  
**Title:** Preemptive Scheduler & Context Switch (Priority Classes, Quantum Preemption, & sched_hint)  
**Date:** 2026-10-26  
**Status:** ✅ **PASS (Sprint Exit Criteria & Scheduler Invariants Satisfied)**  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (INDOS-BIBLE-001 Master §8.8, §9, §24.1), `docs/architecture/scheduler-spec.md` (`INDOS-SPEC-SCHED-01`)  
**Requirements Addressed:** `REQ-KRN-101` (Preemptive Scheduling & Process Life Cycle), `REQ-KRN-102` (Hardware MMU Address Space Isolation), `REQ-KRN-104` (Priority Classes & Workload Governance), `REQ-AI-001` (Advisory sched_hint Interface Bounds)  
**Evidence Pack:** `evidence/SP-12/manifest.json`

---

### 1. Sprint Deliverables

In strict adherence to Development Bible §8.8 (`Phase 2 — Core Kernel: SP-12`), Architecture Specification `INDOS-SPEC-SCHED-01`, and Rule N-01 (zero external kernel crates):

1. **Preemptive Multi-Level Priority Scheduler ([kernel/src/task/scheduler.rs](file:///Users/dhan/IndOS/kernel/src/task/scheduler.rs)):**
   - Implements 5 fixed priority classes:
     - `Priority::Realtime` (4) / `REALTIME_HARDWARE`: Strictly preempts lower classes (timers, audio, critical interrupts).
     - `Priority::Interactive` (3) / `INTERACTIVE_USER`: Interactive UI shell and user-facing processes.
     - `Priority::Normal` (2) / `NORMAL_SERVICES`: Standard system daemons and general services.
     - `Priority::BatchIntelligence` (1) / `BATCH_INTELLIGENCE`: AIS background model inference and batch indexing.
     - `Priority::Idle` (0) / `IDLE`: CPU low-power idle task.
   - Implements multi-level run queues (`[Vec<Process>; 5]`) with strict priority preemption and FIFO/round-robin dispatch within the same priority class.
   - Configures 20ms time slice (`DEFAULT_TIME_QUANTUM_TICKS = 2` ticks at 100 Hz Generic Timer rate).
   - Tracks scheduling metrics: `total_context_switches`, `total_preemptions`, `total_yields`, and `total_hints_received`.

2. **Advisory `sched_hint` Telemetry Interface (Bible §8.8, `INDOS-SPEC-SCHED-01` §3, & Rule N-04):**
   - Implements `SchedHint` enum: `YieldNow` (0), `LatencySensitive` (1), `ThroughputOriented` (2), `BackgroundWork` (3), `AiInferenceBoost` (4).
   - In accordance with **Rule N-04 (Security is enforced, never requested)**, hints submitted by user processes or AI models are strictly advisory stubs that cannot bypass priority ceilings or starve higher-priority tasks.
   - Exposed to user mode via system call `SYS_SCHED_HINT` (syscall number 4).

3. **Process Subsystem Integration ([kernel/src/task/process.rs](file:///Users/dhan/IndOS/kernel/src/task/process.rs) & [kernel/src/task/mod.rs](file:///Users/dhan/IndOS/kernel/src/task/mod.rs)):**
   - Binds `priority: Priority` into the Process Control Block (`Process`).
   - Implements `Scheduler::add_process`, `Scheduler::on_timer_tick`, `Scheduler::schedule_next`, `Scheduler::yield_current`, and `Scheduler::exit_current`.
   - Implements `run_scheduler()` multi-process coordination loop: enters EL0, handles concurrent preemption and yielding across multiple processes, and returns cleanly to kernel mode when all processes complete.

4. **System Call Subsystem Extensions ([kernel/src/syscall/mod.rs](file:///Users/dhan/IndOS/kernel/src/syscall/mod.rs)):**
   - Implements `SYS_SCHED_HINT` (4) calling `SCHEDULER.lock().sched_hint(pid, hint)`.
   - Connects `SYS_YIELD` (0) to `SCHEDULER.lock().yield_current(context)`.
   - Connects `SYS_EXIT` (2) to `SCHEDULER.lock().exit_current(status, context)`.
   - Connects `SYS_GETPID` (3) to active process PID lookup in `SCHEDULER`.

5. **AArch64 Low-Level Preemption & Interrupt Trampolines ([kernel/src/arch/aarch64/user_trap.s](file:///Users/dhan/IndOS/kernel/src/arch/aarch64/user_trap.s) & [kernel/src/arch/aarch64/exceptions.s](file:///Users/dhan/IndOS/kernel/src/arch/aarch64/exceptions.s)):**
   - Routes `vec_lower_el_a64_irq` (`VBAR_EL1 + 0x480`) to `common_lower_el_irq_entry`.
   - Implements `common_lower_el_irq_entry`: allocates 288-byte `UserContext` on the kernel stack, saves `x0..x30`, `sp_el0`, `elr_el1`, `spsr_el1`, and calls `handle_lower_el_irq`.
   - In `handle_lower_el_irq`: services the 100 Hz timer tick, decrements quantum, triggers `schedule_next(ctx)`, activates the new process's `TTBR0_EL1` address space and ASID with pipeline synchronization (`isb`), and restores the new process context on `eret`.
   - Embeds two distinct user-mode test processes (`user_task_alpha` and `user_task_beta`) exercising voluntary yielding (`SYS_YIELD`), advisory hints (`SYS_SCHED_HINT`), work loops with involuntary timer preemption, and clean exit (`SYS_EXIT`).

6. **Full Preemptive Multitasking Verification in QEMU ([kernel/src/main.rs](file:///Users/dhan/IndOS/kernel/src/main.rs)):**
   - Emits boot stage markers `[STAGE:22_SCHEDULER_INIT]`, `[STAGE:23_PREEMPTIVE_MULTITASKING]`, and `[STAGE:24_SCHEDULER_READY]`.
   - Concurrently executes Process 1 (`proc_alpha`, PID 1) and Process 2 (`proc_beta`, PID 2) in isolated per-process address spaces.
   - Verifies 6+ atomic context switches, multiple timer preemptions (5+), voluntary yields (2), and zero corruptions or panics.

---

### 2. Invariant & Safety Verification

| Invariant | Description | Verification Method | Result |
| :--- | :--- | :--- | :---: |
| **INV-SCHED-001** | Atomic Context Switch & Register Integrity | QEMU execution verifying full general-purpose (`x0..x30`), `sp_el0`, and `elr_el1` preservation across preemptions | ✅ **PASS** |
| **INV-SCHED-002** | Priority Classes & Workload Preemption | Verification of 5 priority classes (`REALTIME_HARDWARE` through `IDLE`) and fairness bounds in `main.rs` (Stage 22) | ✅ **PASS** |
| **INV-IRQ-004** | 100 Hz Timer Tick Preemption during EL0 | Hardware timer PPI IRQ 30 servicing during user mode, quantum countdown (2 ticks / 20ms), and preemptive task switch | ✅ **PASS** |
| **INV-MEM-001** | Address Space Isolation during Context Switch | Independent `TTBR0_EL1` and ASID switching per process; separate text and stack mappings at `0x0040_0000` | ✅ **PASS** |
| **Rule N-04** | Advisory `sched_hint` Security Invariant | Security is enforced, never requested: hints logged as non-authoritative telemetry; cannot starve tasks | ✅ **PASS** |
| **Rule N-01** | Zero External Crates | Pure `no_std` Rust + AArch64 assembly with freestanding spinlocks | ✅ **PASS** |
| **Bible §24.1** | Unsafe Budget Monotonic Control | Audited in `docs/engineering/unsafe-budget.md` (`UNSAFE-015`, `UNSAFE-016`) with `// SAFETY:` rationale | ✅ **PASS** |

---

### 3. CI Test Harness & QEMU Golden Log Evidence

The full verification suite was executed via `ci/verify_sp12.py`:
- Deliverables check: **8/8 present**.
- Architecture specification `INDOS-SPEC-SCHED-01`: **Verified**.
- Scheduler symbols & multi-level queues: **Verified**.
- System call ABI (`SYS_SCHED_HINT`, `SYS_YIELD`, `SYS_EXIT`): **Verified**.
- Unsafe-code budget scanner (`ci/check_unsafe_budget.py`): **16/16 entries audited, 0 unannotated blocks**.
- Kernel build: **Zero warnings, zero errors**.
- Headless QEMU execution against [docs/evidence/golden-boot-sp12.log](file:///Users/dhan/IndOS/docs/evidence/golden-boot-sp12.log): **100% Match**.
- Evidence manifest: [evidence/SP-12/manifest.json](file:///Users/dhan/IndOS/evidence/SP-12/manifest.json).

---

### 4. Sprint Status: Complete

Sprint SP-12 has successfully achieved its Definition of Done. The Ind OS kernel (`IndK`) now possesses a fully functional, priority-aware, hardware-preemptive multi-tasking scheduler with atomic context switching, timer quantum management, advisory hints, and address space isolation.
