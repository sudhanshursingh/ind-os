# Spike Record: QEMU Boot Path Trade-off (Direct `-kernel` vs. U-Boot)
**Document ID:** `INDOS-SPIKE-BOOT-01`  
**Date:** 2026-10-26  
**Status:** RESOLVED / RATIFIED  
**Governing Sprint:** Phase 0, Sprint SP-01  
**Target:** ARM64 QEMU `virt` platform $\rightarrow$ Moto G60 (`hanoip`)

---

## 1. Context & Objective

In bringing up `IndK` on the ARM64 QEMU `virt` platform, two primary boot-loading methodologies exist:
1. **Option 1: Direct QEMU `-kernel` ELF execution.**
   - QEMU loads the uncompressed 64-bit ELF binary directly into RAM at physical address `0x40080000`, initializes CPU register `x0` with the pointer to the generated Device Tree Blob (DTB), and jumps directly to the entry point in Exception Level 1 (EL1) or EL2.
2. **Option 2: Firmware / U-Boot intermediary.**
   - QEMU boots into U-Boot or TF-A (Trusted Firmware-A), which initializes simulated peripherals, sets up environment variables, and parses an image format (FIT or legacy uImage) before handing control to the kernel.

---

## 2. Technical Comparison & Trade-off Matrix

| Metric | Option 1: Direct QEMU `-kernel` | Option 2: U-Boot / Firmware Intermediary |
| :--- | :--- | :--- |
| **Boot Latency to Entry** | **< 10 ms** (instant execution) | ~1.5 - 2.5 seconds (firmware POST/timeouts) |
| **TCB & External Code** | **Zero firmware dependency.** Completely clean boundary. | Requires complex U-Boot C codebase, U-Boot scripts, and additional toolchains. |
| **Rule N-01 Compliance** | **100% clean.** No foreign bootloader binaries. | Introduces thousands of lines of external GPL firmware code. |
| **Cold Boot Reproducibility** | **Deterministic.** QEMU sets registers `x0..x3` reliably per standard Linux/ARM64 boot protocol. | Non-deterministic across U-Boot version variations and environment states. |
| **Physical Target Parity** | High conceptual parity with custom LK/ABL Android boot chain on Qualcomm. | High parity with standard embedded Linux dev boards. |

---

## 3. Decision

> **Decision:** We select **Option 1: Direct QEMU `-kernel` ELF execution** as the primary development boot path for Phase 0–6 on ARM64 QEMU `virt`.

### Architectural Rules for `IndK` Boot Entry:
1. The kernel linker script (`kernel/arch/aarch64/linker.ld`) places the boot entry point at `0x40080000` (standard QEMU RAM base).
2. The boot assembly (`kernel/arch/aarch64/boot.s`) expects `x0` to hold the physical address of the device tree blob (DTB).
3. The kernel validates the DTB magic number (`0xd00dfeed`) immediately upon entry.
4. When migrating to physical hardware in Phase 9 (`hanoip`), a thin freestanding boot shim (per Section 6.2) adapts this same convention from Qualcomm's Android Bootloader (ABL).
