# Ind OS Sprint SP-11 Validation Report
**Sprint ID:** SP-11  
**Phase:** Phase 2 — Core Kernel  
**Title:** Processes & Loading (PCB, INDBIN Native Loader, Real EL0 User Mode, & SVC Syscalls)  
**Date:** 2026-10-26  
**Status:** ✅ **PASS (Sprint Exit Criteria & Process Invariants Satisfied)**  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (INDOS-BIBLE-001 Master §8.8, §9, §24.1), `docs/DECISIONS/ADR-001-binary-format.md`  
**Requirements Addressed:** `REQ-KRN-101` (Process Abstraction & User-Mode Isolation), `REQ-KRN-102` (Virtual Memory Mapping & MMU), `REQ-KRN-103` (Memory Protection & Privilege Isolation), `REQ-SEC-101` (Default-Deny User-Pointer Sanitization)  
**Evidence Pack:** `evidence/SP-11/manifest.json`

---

### 1. Sprint Deliverables

In strict adherence to Development Bible §8.8 (`Phase 2 — Core Kernel: SP-11`), ADR-001, and Rule N-01 (zero external kernel crates):

1. **Architectural Decision Record ([docs/DECISIONS/ADR-001-binary-format.md](file:///Users/dhan/IndOS/docs/DECISIONS/ADR-001-binary-format.md)):**
   - Formally evaluated executable binary formats for Ind OS.
   - Adopted hybrid strategy: Hardened, native micro-executable format (`INDBIN`, magic `0x42444E49`) in kernel space to eliminate TCB parser complexity, paired with standard host toolchain conversion from static AArch64 ELF64 images.
   - Recorded in [docs/DECISIONS/register.md](file:///Users/dhan/IndOS/docs/DECISIONS/register.md).

2. **INDBIN Native Binary Format & Loader ([kernel/src/loader/indbin.rs](file:///Users/dhan/IndOS/kernel/src/loader/indbin.rs)):**
   - Implements 80-byte fixed header parser (`IndbinHeader`) specifying entry point, text/data virtual addresses, offsets, sizes, and user stack requirements.
   - Validates memory invariants at load time: rejects executables targeting the NULL trap zone (`0x0000_0000 .. 0x0040_0000`, `INV-MEM-001`) or outside user space (`USER_SPACE_END`).
   - Allocates dedicated physical page frames from `FRAME_ALLOCATOR`, zeroes memory, copies segment payloads, and binds user page table entries with strict permission bits:
     - Text segment: `PageTableFlags::USER_CODE` (Read-Only, `PTE_PXN = 1` preventing EL1 execution of user code).
     - Data & Stack segments: `PageTableFlags::USER_DATA` (Read/Write, `PTE_PXN = 1`, `PTE_UXN = 1`).
   - Anchors 16KB user stack at `USER_STACK_TOP = 0x0000_7FFF_FFFF_0000` growing downwards.

3. **Process Abstraction & Control Block ([kernel/src/task/process.rs](file:///Users/dhan/IndOS/kernel/src/task/process.rs)):**
   - Implements `ProcessControlBlock` (`Process`): PID, process name, lifecycle state (`ProcessState`: `Ready`, `Running`, `Blocked`, `Exited(i32)`), per-process `AddressSpace`, and `UserContext`.
   - Defines strictly 16-byte aligned `UserContext` (288 bytes) capturing general-purpose registers `x0..x30`, `sp_el0`, `elr_el1`, `spsr_el1`, and `esr_el1`.
   - Defines callee-saved `KernelContext` for saving and restoring kernel continuation across user execution trampolines.

4. **System Call Subsystem ([kernel/src/syscall/mod.rs](file:///Users/dhan/IndOS/kernel/src/syscall/mod.rs)):**
   - Implements AArch64 Register ABI dispatch:
     - `x8`: System call number (`SYS_YIELD = 0`, `SYS_WRITE = 1`, `SYS_EXIT = 2`, `SYS_GETPID = 3`).
     - `x0..x5`: System call arguments.
     - `x0`: Return code.
   - Implements default-deny user pointer sanitizer (`validate_user_buffer`): rejects pointers in the NULL trap zone (`INV-MEM-001`) or outside `USER_SPACE_END` with `-EFAULT` (`-14`).
   - Connects `SYS_WRITE` to console output and `SYS_EXIT` to process termination.

5. **AArch64 Low-Level User Mode Entry, Exit & Trap Trampolines ([kernel/src/arch/aarch64/user_trap.s](file:///Users/dhan/IndOS/kernel/src/arch/aarch64/user_trap.s)):**
   - Implements `kernel_call_user`: preserves kernel callee-saved registers (`x19..x30`, `sp`), loads `sp_el0`, `elr_el1`, `spsr_el1 = 0x0` (EL0t mode), restores user registers, and drops privilege from EL1 to EL0 via `eret`.
   - Implements `common_lower_el_sync_entry`: preserves user registers upon `svc #0` synchronous traps, passes context to Rust `handle_lower_el_sync`, dispatches system call, and returns to EL0 via `eret`.
   - Implements `kernel_return_from_user`: called upon `SYS_EXIT` to cleanly restore kernel execution state and return exit status to `run_user_process`.
   - Routes `vec_lower_el_a64_irq` to `common_irq_entry` in [exceptions.s](file:///Users/dhan/IndOS/kernel/src/arch/aarch64/exceptions.s), ensuring timer ticks are serviced transparently during EL0 execution (`INV-IRQ-004`).

6. **First Real EL0 User-Mode Process Execution ([kernel/src/main.rs](file:///Users/dhan/IndOS/kernel/src/main.rs)):**
   - Instantiates Process 1 ("hello_user") from embedded AArch64 machine code (`HELLO_USER_CODE`).
   - Drops to AArch64 EL0 user mode with hardware MMU active (`SCTLR_EL1.M = 1`).
   - User code executes:
     ```
     Hello from Ind OS User Mode (EL0)!
     ```
   - User code calls `sys_exit(0)`, returning cleanly to EL1 kernel execution.
   - Emits boot stage markers `[STAGE:19_PROCESS_SUBSYSTEM_INIT]`, `[STAGE:20_USER_MODE_HELLO]`, and `[STAGE:21_SYSCALL_DISPATCH_READY]`.

---

### 2. Invariant Verification Audit

| Invariant ID | Specification Statement | Verification Mechanism | Status |
| :--- | :--- | :--- | :---: |
| **INV-MEM-001** | Strict NULL-pointer trap zone (`0x0000_0000 .. 0x0040_0000`) strictly unmapped | Verified in loader: test image targeting `0x0000_1000` is rejected with `LoaderError::NullTrapViolation`. Syscall buffer validator rejects addresses `< 0x0040_0000` with `-EFAULT`. | ✅ **PASS** |
| **INV-MEM-002** | PXN and UXN execution boundaries strictly enforced between EL1 kernel and EL0 user | User code mapped with `USER_CODE` (`PTE_PXN = 1`), user stack/data mapped with `USER_DATA` (`PTE_PXN = 1`, `PTE_UXN = 1`), kernel code mapped with `PTE_UXN = 1`. | ✅ **PASS** |
| **INV-BOOT-002** | Stack pointer is strictly 16-byte aligned at all times | Kernel stack and user stack (`USER_STACK_TOP = 0x00007fffffff0000`) adhere to 16-byte alignment. Context frames are multiples of 16 bytes. | ✅ **PASS** |
| **INV-IRQ-004** | Periodic hardware timer ticks (100 Hz) serviced without corrupting user context | `vec_lower_el_a64_irq` routed to `common_irq_entry`; preserves `x0..x30`, updates tick counter, and returns via `eret`. | ✅ **PASS** |

---

### 3. Automated Verification Results

Execution of `python3 ci/verify_sp11.py`:

```
==================================================
 Ind OS SP-11 Processes & Loading Verifier       
 Target: Sprint SP-11 Definition of Done (DoD)   
==================================================
  [OK] Deliverable exists: docs/DECISIONS/ADR-001-binary-format.md
  [OK] Deliverable exists: kernel/src/loader/indbin.rs
  [OK] Deliverable exists: kernel/src/loader/mod.rs
  [OK] Deliverable exists: kernel/src/task/process.rs
  [OK] Deliverable exists: kernel/src/task/mod.rs
  [OK] Deliverable exists: kernel/src/syscall/mod.rs
  [OK] Deliverable exists: kernel/src/arch/aarch64/user_trap.s
  [OK] Deliverable exists: docs/evidence/golden-boot-sp11.log
  [OK] ADR-001 (INDBIN & ELF64 strategy) recorded in decision register.
  [OK] INDBIN binary format specification & loader verified.
  [OK] Process Control Block and context structures verified.
  [OK] System call ABI and user pointer validation verified.
  [OK] Invariant INV-MEM-001 (NULL trap zone enforcement) verified.
  [OK] AArch64 user mode entry, return, and lower EL sync traps verified.
• Validating Unsafe-Code Budget adherence (Bible §24.1)...
  [OK] Unsafe-code budget scanner passed cleanly.
• Building IndK ARM64 kernel...
  [OK] Kernel compiled cleanly with zero errors.
  [OK] Kernel binary produced: kernel/target/aarch64-unknown-none/debug/indk.bin
• Executing QEMU test harness against golden-boot-sp11.log...
  [OK] QEMU boot execution matched golden reference exactly (100% PASS).
  [OK] Generated evidence manifest: evidence/SP-11/manifest.json
==================================================
✅ Sprint SP-11 Verification: 100% PASSED
==================================================
```

---

### 4. Machine State at Sprint Exit

- **Active Process:** Process 1 ("hello_user") created, executed in EL0, and transitioned to `ProcessState::Exited(0)`.
- **Hardware Mode:** AArch64 EL0 execution and clean exception return to EL1 verified under hardware MMU translation.
- **Syscall Handling:** `SYS_WRITE` and `SYS_EXIT` operational with register ABI (`x8`, `x0..x5`).
- **Unsafe-Code Budget:** Audited per Section 24 with entries `UNSAFE-013` and `UNSAFE-014` registered and verified.
