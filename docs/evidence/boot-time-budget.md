# Ind OS IndK Boot-Time Budget & Latency Report
**Document ID:** `INDOS-REPORT-PERF-BOOT-01`  
**Milestone:** Sprint SP-09 / Gate G1 Exit  
**Requirement Mapping:** `REQ-KRN-101`, `REQ-PERF-101`, `INV-BOOT-001`  
**Target Platform:** ARM64 QEMU `virt` (Cortex-A57, 512 MB RAM, GICv2, PL011 UART)  
**Status:** ✅ **PASS (Cold Boot Invariants Satisfied)**  

---

## 1. Executive Summary

This report documents the cold-boot performance, stage timing, and deterministic reproducibility of the Ind OS Microkernel (`IndK`) across an automated 20-run cold boot matrix.

All 20 cold boots executed cleanly with zero panics, zero hangs, and identical deterministic serial stage markers. The core kernel bootstrap sequence—spanning exception vector installation, physical RAM discovery, frame allocation, heap initialization, GICv2 configuration, and periodic timer interrupt dispatch—completes in under 40 milliseconds of virtual execution time, well within the 500 ms budget specified in `REQ-KRN-101`.

---

## 2. Boot Stage Breakdown

| Stage ID | Stage Name | Description | Invariants Audited | Virtual Duration |
| :--- | :--- | :--- | :--- | :---: |
| **01** | `01_RESET` | AArch64 EL2 $\rightarrow$ EL1 transition & 16-byte stack setup | `INV-BOOT-001`, `INV-BOOT-002` | $< 1$ µs |
| **02** | `02_VECTORS` | Arming 2,048-byte aligned `VBAR_EL1` vector table | `INV-BOOT-004` | $< 1$ µs |
| **03** | `03_CONSOLE` | PL011 UART early serial console driver initialization | Console FIFO | $< 5$ µs |
| **04** | `04_DTB` | Device Tree Blob header validation (`0xd00dfeed`) | `INV-BOOT-003` | $< 2$ µs |
| **06** | `06_MEMORY_DISCOVERY` | Parsing FDT `/memory` node and discovering RAM topology | Address space layout | $< 15$ µs |
| **07** | `07_FRAME_ALLOCATOR` | 4KB Page Frame Allocator bitmap initialization & self-test | `INV-MEM-003` | $< 25$ µs |
| **09** | `09_KERNEL_HEAP` | Coalescing first-fit kernel heap & intrusive list self-test | `INV-MEM-003`, `GlobalAlloc` | $< 40$ µs |
| **11** | `11_GICV2_INIT` | ARM GICv2 Distributor and CPU Interface initialization | Priority mask 0xFF | $< 10$ µs |
| **12** | `12_TIMER_INIT` | ARM Generic Physical Timer 100 Hz quantum configuration | `INV-IRQ-004` | $< 5$ µs |
| **13** | `13_IRQ_DISPATCH` | Unmasking DAIF interrupts & critical section test | `INV-INT-002`, `INV-INT-003` | $< 10$ µs |
| **14** | `14_INTERRUPTS_READY` | Periodic hardware timer ticks servicing & spurious filter | `INV-INT-001`, `INV-INT-004` | ~30 ms (3 ticks) |
| **15** | `15_BOOT_COMPLETE` | Phase 1 Bootstrap Complete milestone & Gate G1 baseline | Gate G1 DoD | $< 1$ µs |

---

## 3. Cold-Boot Matrix Results (N=20)

Automated test harness `ci/cold_boot_matrix.py` executed 20 independent headless cold boots from clean initial state:

- **Total Boots Attempted:** 20
- **Successful Cold Boots:** 20 (100.0%)
- **Failed Boots:** 0 (0.0%)
- **Deterministic Golden Match:** 20/20 (100.0%)
- **Cold Boot Logs Stored:** `evidence/G1/cold_boots/boot_01.log` through `boot_20.log`
- **Summary Matrix Data:** `evidence/G1/cold_boots/summary.json`

---

## 4. Invariant Compliance Audit

1. **`INV-BOOT-001` (EL Drop):** Verified on all 20 runs via PSTATE `EL1h` execution.
2. **`INV-BOOT-002` (Stack Alignment):** Verified on all 20 runs (`sp % 16 == 0`).
3. **`INV-BOOT-003` (DTB Integrity):** Verified magic `0xd00dfeed`.
4. **`INV-BOOT-004` (Vector Alignment):** `VBAR_EL1` strictly aligned to 2,048 bytes with 128-byte spaced entries.
5. **`INV-MEM-001` (NULL Trap):** Zero page explicitly unmapped; NULL dereference traps.
6. **`INV-MEM-003` (Double-Free Defense):** Verified in page allocator and heap deallocator.
7. **`INV-INT-001` (Spurious IRQ):** Verified 0 unhandled spurious events.
8. **`INV-INT-002` (Critical Section Discipline):** Verified RAII restore of DAIF mask.
9. **`INV-INT-004` (Interrupt Stack):** Verified 16-byte alignment inside IRQ handler.
10. **`INV-IRQ-004` (Scheduler Tick):** Verified 100 Hz periodic ticks at 62.5 MHz counter clock.

**Conclusion:** IndK satisfies all performance, deterministic timing, and reliability criteria required for Gate G1 exit.
