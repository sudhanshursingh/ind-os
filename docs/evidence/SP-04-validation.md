# Ind OS Sprint SP-04 Validation Report
**Sprint ID:** SP-04  
**Phase:** Phase 1 — Kernel Bootstrap  
**Title:** Boot Entry & Exception Vectors  
**Date:** 2026-10-26  
**Status:** ✅ **PASS (Sprint Exit Criteria & Machine Invariants Satisfied)**  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (INDOS-BIBLE-001 Master)  
**Requirements Addressed:** `REQ-KRN-101` (AArch64 bare-metal target), `REQ-KRN-102` (EL2 to EL1 drop & HCR_EL2 config)  
**Evidence Pack:** `evidence/SP-04/manifest.json`

---

### 1. Sprint Deliverables

In strict adherence to Development Bible §8.7 (`Phase 1 — Kernel Bootstrap: SP-04`) and Rule N-01 (Zero external kernel crates, pure assembly and `no_std` Rust):

1. **Kernel Crate Scaffolding ([kernel/Cargo.toml](file:///Users/dhan/IndOS/kernel/Cargo.toml)):**
   - Pure `no_std`, `no_main` freestanding Rust crate (`indk`). Zero external crate dependencies.
   - Pinned to toolchain `1.82.0-aarch64-unknown-none`.

2. **QEMU Virt Linker Script ([kernel/linker.ld](file:///Users/dhan/IndOS/kernel/linker.ld)):**
   - Targets QEMU ARM64 `virt` RAM origin at physical address `0x40080000`.
   - Organizes `.text.boot` at origin, followed by `.text`, strictly aligned `.vectors` (2,048 bytes), `.rodata`, `.data`, `.bss`, and a 64 KB 16-byte aligned early kernel stack (`__stack_bottom` to `__stack_top`).

3. **Reset & Boot Routine ([kernel/src/arch/aarch64/boot.s](file:///Users/dhan/IndOS/kernel/src/arch/aarch64/boot.s)):**
   - Preserves DTB pointer passed in `x0` upon reset into `x19`.
   - Inspects `CurrentEL`. If in EL2, configures `HCR_EL2.RW = 1` (AArch64 EL1 execution), timer access in `CNTHCTL_EL2`, and executes `eret` to drop to EL1h.
   - Transitions to EL1 within 19 instructions (satisfying `INV-BOOT-001` $\le 50$ instructions).
   - Initializes SP to `__stack_top` and asserts 16-byte alignment via `tst x1, #0xf` (satisfying `INV-BOOT-002`). Faults loudly to UART if misaligned.
   - Programs `VBAR_EL1` with `__vector_table`.
   - Zero-clears the `.bss` section.
   - Validates the Device Tree Blob pointer and asserts magic `0xd00dfeed` (satisfying `INV-BOOT-003`).

4. **Exception Vector Table ([kernel/src/arch/aarch64/exceptions.s](file:///Users/dhan/IndOS/kernel/src/arch/aarch64/exceptions.s)):**
   - Defines `__vector_table` aligned to 2,048 bytes (`.align 11`).
   - Contains all 16 architectural AArch64 exception vectors, spaced at exactly 128 bytes (`.align 7`, offset $0\text{x}80 \times i$) per `INV-BOOT-004`.
   - Saves complete context (31 general-purpose registers, `ELR_EL1`, `SPSR_EL1`, `ESR_EL1`, `FAR_EL1`) onto the stack before dispatching to Rust exception handler.

5. **Freestanding Entry & Panic Dumper ([kernel/src/main.rs](file:///Users/dhan/IndOS/kernel/src/main.rs), [kernel/src/panic.rs](file:///Users/dhan/IndOS/kernel/src/panic.rs)):**
   - Freestanding `kernel_main(dtb_addr: usize)` validates DTB header values.
   - Structured `EarlyUart` driver for ARM PL011 at physical base `0x09000000`.
   - Comprehensive `#[panic_handler]` emitting structured panic dumps over serial.

6. **Golden Boot Log ([docs/evidence/golden-boot-sp04.log](file:///Users/dhan/IndOS/docs/evidence/golden-boot-sp04.log)):**
   - Authoritative milestone sequence log for automated verification.

---

### 2. Invariant Verification Audit

| Invariant ID | Specification Statement | Verification Mechanism | Status |
| :--- | :--- | :--- | :---: |
| **INV-BOOT-001** | Kernel executes in EL1 within $\le 50$ instructions of entry | Static assembly instruction count: 19 instructions from `_start` through `eret` to `Lin_el1`. | ✅ **PASS** |
| **INV-BOOT-002** | Early kernel stack pointer strictly 16-byte aligned | Checked in `boot.s` (`tst x1, #0xf`, branches to `.Lstack_misaligned_panic` on fault) and verified in `kernel_main`. | ✅ **PASS** |
| **INV-BOOT-003** | `x0` preserves DTB pointer and validates magic `0xd00dfeed` | Checked in `boot.s` (`0xd00dfeed` rev check, branches to `.Ldtb_corrupted_panic` on fault) and printed during boot. | ✅ **PASS** |
| **INV-BOOT-004** | Exception vector table `VBAR_EL1` aligned to 2,048 bytes with 16 128-byte spaced entries | ELF symbol inspection confirms `__vector_table` at `0x40084000` (`% 2048 == 0`) and all 16 entries spaced by exactly `0x80` bytes. | ✅ **PASS** |

---

### 3. Automated Verification Results

Execution of `python3 ci/verify_sp04.py`:

```text
==================================================
 Ind OS SP-04 Boot Entry & Vector Verifier       
 Target: Sprint SP-04 Definition of Done (DoD)   
==================================================
  [OK] Deliverable exists: kernel/Cargo.toml
  [OK] Deliverable exists: kernel/linker.ld
  [OK] Deliverable exists: kernel/src/main.rs
  [OK] Deliverable exists: kernel/src/panic.rs
  [OK] Deliverable exists: kernel/src/arch/mod.rs
  [OK] Deliverable exists: kernel/src/arch/aarch64/mod.rs
  [OK] Deliverable exists: kernel/src/arch/aarch64/boot.s
  [OK] Deliverable exists: kernel/src/arch/aarch64/exceptions.s
  [OK] Deliverable exists: docs/evidence/golden-boot-sp04.log
  [OK] INV-BOOT-001: EL drop & EL1 transition sequence present.
  [OK] INV-BOOT-002: 16-byte stack alignment assertion present.
  [OK] INV-BOOT-003: DTB pointer & 0xd00dfeed magic validation present.
  [OK] INV-BOOT-004: All 16 vector entries defined with .align 11 table header.
• Compiling IndK ARM64 kernel...
  [OK] IndK compiled cleanly.
  [OK] Generated raw kernel image: kernel/target/aarch64-unknown-none/debug/indk.bin (21848 bytes)
  [OK] __vector_table is strictly 2,048-byte aligned at: 0x40084000
  [OK] All 16 vector entries spaced exactly at 0x80 (128 bytes) intervals.
• Running QEMU test via ci/run_qemu_test.py...
  [OK] Headless QEMU execution passed and serial log matches golden reference.
  [OK] Generated evidence manifest: evidence/SP-04/manifest.json
==================================================
 ✅ SPRINT SP-04 VERIFICATION PASSED (100% GREEN) 
==================================================
```

Execution of `make check`:
- **SP-00 Verifier:** PASS
- **SP-01 Verifier:** PASS
- **SP-02 Verifier:** PASS
- **SP-03 Verifier:** PASS
- **SP-04 Verifier:** PASS
- **Total Test Suites:** 5/5 Green (100%).
