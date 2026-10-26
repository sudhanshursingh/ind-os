# Ind OS Sprint SP-05 Validation Report
**Sprint ID:** SP-05  
**Phase:** Phase 1 — Kernel Bootstrap  
**Title:** Serial Console & Leveled Logging  
**Date:** 2026-10-26  
**Status:** ✅ **PASS (Sprint Exit Criteria & Serial Console Requirements Satisfied)**  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (INDOS-BIBLE-001 Master §8.7)  
**Requirements Addressed:** `REQ-KRN-101` (IndK owns serial console on ARM64 QEMU virt), Gate G1 Build Provenance Standard  
**Evidence Pack:** `evidence/SP-05/manifest.json`

---

### 1. Sprint Deliverables

In strict adherence to Development Bible §8.7 (`Phase 1 — Kernel Bootstrap: SP-05`), Rule N-01 (zero external kernel crates), and Rule N-10 (Requirement referencing):

1. **Build Provenance Embedder ([kernel/build.rs](file:///Users/dhan/IndOS/kernel/build.rs)):**
   - Automatically extracts and compiles the current Git commit hash, target architecture (`aarch64`), and target platform (`qemu-virt`) directly into the kernel binary.
   - Satisfies the Gate G1 audit requirement: *"serial log uniquely identifies the build (git hash + timestamp + config hash)"*.

2. **ARM PrimeCell PL011 UART Driver ([kernel/src/drivers/pl011.rs](file:///Users/dhan/IndOS/kernel/src/drivers/pl011.rs)):**
   - Full register-level driver for the ARM PL011 UART at physical address `0x09000000`.
   - Reprograms control registers (`UARTCR`), clears interrupts (`UARTICR`), calculates and sets baud rate divisors for 115,200 baud (`UARTIBRD=13`, `UARTFBRD=2`), enables 32-byte hardware FIFOs and 8N1 word length (`UARTLCR_H`).
   - Implements polled non-blocking reception (`read_byte`), transmit FIFO full detection (`is_tx_full`), and implements `core::fmt::Write`.

3. **Freestanding Spinlock Primitive ([kernel/src/sync/spinlock.rs](file:///Users/dhan/IndOS/kernel/src/sync/spinlock.rs)):**
   - Implements mutual exclusion using atomic test-and-set (`AtomicBool` with `Acquire`/`Release` semantics) and exponential backoff (`core::hint::spin_loop()`).
   - Zero heap allocation, fully `no_std` compatible, safe RAII guard pattern (`SpinlockGuard`).

4. **Console Subsystem ([kernel/src/console.rs](file:///Users/dhan/IndOS/kernel/src/console.rs)):**
   - Thread-safe `CONSOLE` singleton wrapping `Spinlock<Pl011Uart>`.
   - Exported `print!` and `println!` macros supporting standard Rust formatting arguments (`format_args!`).

5. **Leveled Logging & Boot-Stage Marker Framework ([kernel/src/logger.rs](file:///Users/dhan/IndOS/kernel/src/logger.rs)):**
   - Defines hierarchical log levels: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` with compile-time and runtime filtering.
   - Exported logging macros: `log_trace!`, `log_debug!`, `log_info!`, `log_warn!`, `log_error!`.
   - Implements deterministic boot-stage transition markers via `stage_marker!(stage_id, description)` emitting formatted `[STAGE:<id>] <desc>` lines.

6. **Enhanced Freestanding Main Entry Point ([kernel/src/main.rs](file:///Users/dhan/IndOS/kernel/src/main.rs)):**
   - Initializes PL011 serial console immediately after EL1 stack entry.
   - Prints authoritative build provenance banner.
   - Emits structured stage markers (`01_RESET`, `02_VECTORS`, `03_CONSOLE`, `04_DTB`, `05_READY`).
   - Validates DTB header fields and dynamically reads device tree size.

7. **Golden Boot Log ([docs/evidence/golden-boot-sp05.log](file:///Users/dhan/IndOS/docs/evidence/golden-boot-sp05.log)):**
   - Authoritative ordered reference pattern supporting regex matching of dynamic build hashes and DTB memory pointers.

---

### 2. Verification Results

Execution of `python3 ci/verify_sp05.py`:

```text
==================================================
 Ind OS SP-05 Serial Console & Logger Verifier   
 Target: Sprint SP-05 Definition of Done (DoD)   
==================================================
  [OK] Deliverable exists: kernel/build.rs
  [OK] Deliverable exists: kernel/src/drivers/mod.rs
  [OK] Deliverable exists: kernel/src/drivers/pl011.rs
  [OK] Deliverable exists: kernel/src/sync/mod.rs
  [OK] Deliverable exists: kernel/src/sync/spinlock.rs
  [OK] Deliverable exists: kernel/src/console.rs
  [OK] Deliverable exists: kernel/src/logger.rs
  [OK] Deliverable exists: kernel/src/main.rs
  [OK] Deliverable exists: kernel/src/panic.rs
  [OK] Deliverable exists: docs/evidence/golden-boot-sp05.log
  [OK] PL011 driver register definitions, FIFO handling, and fmt::Write verified.
  [OK] Freestanding AtomicBool Spinlock mutual exclusion verified.
  [OK] Thread-safe CONSOLE singleton, print!, and println! macros verified.
  [OK] Leveled logging framework and boot-stage markers verified.
• Compiling IndK ARM64 kernel...
  [OK] IndK compiled cleanly.
  [OK] Generated raw kernel image: kernel/target/aarch64-unknown-none/debug/indk.bin (28696 bytes)
• Running QEMU test via ci/run_qemu_test.py...
  [OK] Headless QEMU execution passed and serial log matches golden reference.
  [OK] Generated evidence manifest: evidence/SP-05/manifest.json
==================================================
 ✅ SPRINT SP-05 VERIFICATION PASSED (100% GREEN) 
==================================================
```

Execution of `make check`:
- **SP-00 Verifier:** PASS
- **SP-01 Verifier:** PASS
- **SP-02 Verifier:** PASS
- **SP-03 Verifier:** PASS
- **SP-04 Verifier:** PASS
- **SP-05 Verifier:** PASS
- **Total Test Suites:** 6/6 Green (100%).
