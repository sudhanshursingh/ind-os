# IndK Invariants Reference Register (Phase 0–2 Baseline)
**Document ID:** `INDOS-INV-REG-01`  
**Governing Rule:** Rule N-02 (Evidence or it didn't happen) & Rule N-10 (Tests with code)  
**Status:** APPROVED FOR PHASE 1–2 IMPLEMENTATION

Every code commit in Phase 1 and 2 must verify compliance against this consolidated invariants register.

| ID | Subsystem | Invariant Statement | Verification Method |
| :--- | :--- | :--- | :--- |
| **INV-BOOT-001** | Boot | Kernel executes in EL1 within $\le 50$ instructions of entry | Register check in QEMU trace log |
| **INV-BOOT-002** | Boot | Stack pointer `sp` is strictly 16-byte aligned | Runtime assertion in `_start` |
| **INV-BOOT-003** | Boot | `x0` holds valid DTB pointer with magic `0xd00dfeed` | `dtb_parser::validate_header()` |
| **INV-BOOT-004** | Boot | Unexpected early exceptions trigger formatted serial panic | Headless QEMU crash test |
| **INV-MEM-001** | Memory | Physical & virtual address `0x0` unmapped (null trap) | Kernel unit test accessing address 0 |
| **INV-MEM-002** | Memory | Kernel/User execution separation (`PXN` and `UXN`) | MMU page table permission audit |
| **INV-MEM-003** | Memory | Page frame allocations audited (zero double-free) | Frame allocator test harness |
| **INV-MEM-004** | Memory | Kernel page tables read-only post-init | Page permission bit audit |
| **INV-IRQ-001** | Interrupts | IRQ dispatch latency $\le 10$ µs on QEMU virt | Timer timestamp delta measurement |
| **INV-IRQ-002** | Interrupts | Nested IRQ disable depth tracked (no early enable) | Spinlock debug tracking |
| **INV-IRQ-003** | Interrupts | Spurious IRQ 1023 filtered without panic | Injected spurious IRQ test |
| **INV-IRQ-004** | Interrupts | Deterministic 100 Hz timer tick configured | Timer frequency cycle count test |
| **INV-SCHED-001**| Scheduler | Context switch time $\le 2.5$ µs | Benchmark loop (1,000 switches) |
| **INV-SCHED-002**| Scheduler | Realtime class strictly non-blocking | Priority inheritance unit test |
| **INV-SCHED-003**| Scheduler | 4 KB unmapped guard page on all thread stacks | Stack overflow fault test |
| **INV-SCHED-004**| Scheduler | AIS batch preemption within $\le 1$ ms on user input | Preemption latency benchmark |
