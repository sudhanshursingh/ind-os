# IndK Preemptive Scheduler & Workload Governance Architecture Specification
**Document ID:** `INDOS-SPEC-SCHED-01`  
**Phase Target:** Phase 2 (SP-12) & Phase 6 (SP-47) $\rightarrow$ Gates G2, G4, G6  
**Requirement Mapping:** `REQ-KRN-104`, `REQ-AI-001`, `REQ-AI-105`  
**Platform:** ARM64 Preemptive Multitasking

---

## 1. Scope & Objectives

This specification defines the priority-based preemptive scheduler for `IndK`, process state lifecycles, context-switching mechanics, and the advisory `sched_hint` telemetry interface governing intelligence subsystem interaction.

---

## 2. Priority Classes & Fairness Discipline

IndK implements four fixed priority classes:

```text
Priority Class 0 (Highest) : REALTIME_HARDWARE (Timers, Audio Streams, Interrupt Bottom-Halves)
                                ▲
                                │ Strictly preempts lower classes
Priority Class 1            : INTERACTIVE_USER (Shell, Consent UI, Touch Input)
                                ▲
                                │ Preempts background processing
Priority Class 2            : NORMAL_SERVICES (Storage, Networking, System Daemons)
                                ▲
                                │ Round-robin quantum (10 ms)
Priority Class 3 (Lowest)  : BATCH_INTELLIGENCE (AIS Model Inference, Log Indexing)
```

### 2.1 Context Switch Sequence
1. Save caller-saved and callee-saved registers (`x19..x28`, `x29`, `x30`) to the current thread's stack.
2. Save current stack pointer into thread control block (`Thread::saved_sp`).
3. Switch `TTBR0_EL1` to new process page tables (with `ISB` instruction barrier and TLB invalidation if ASID differs).
4. Restore registers and return via `eret` or register restore.

---

## 3. The Advisory `sched_hint` Interface (Section 6.4 & Rule N-04)

In accordance with **Principle P2** and **Rule N-04 (Security is enforced, never requested)**:
- The Adaptive Intelligence Subsystem (AIS) may submit advisory hints to the kernel scheduler (e.g., predicting that an interactive task is pending or recommending pre-warming a memory buffer).
- **The Invariant:** All hints are strictly advisory. The kernel evaluates hints inside hard fairness bounds.
- Model output cannot:
  - Starve higher-priority interactive or realtime tasks.
  - Lower the priority of the Consent Broker or UI shell.
  - Exceed the CPU quantum allocated to its own task-scoped capability token.

---

## 4. Invariants (Machine-Verifiable)

- **INV-SCHED-001:** Context switch execution time between two running threads is $\le 2.5$ µs on ARM64 QEMU `virt`.
- **INV-SCHED-002:** The `REALTIME_HARDWARE` class is strictly non-blocking: a lower-priority task never locks out a realtime thread (priority inversion protection via priority inheritance).
- **INV-SCHED-003:** Thread stack overflow is caught by a dedicated 4 KB unmapped guard page at the base of every thread stack.
- **INV-SCHED-004:** AIS background inference processes are immediately preempted within $\le 1$ ms upon the arrival of user interaction events.
