# Ind OS Architecture Specification: Moto G60 Cold-Boot Stability & Reproducibility

**Sprint:** SP-107  
**Milestone:** Phase 9 (Moto G60 Bring-Up)  
**Constitutional References:**
- `Ind_OS_Development_Bible_v1.0.0.0.md` §6.3, §8.15, Table 19.2 (Gate G9 criteria)
- REQ-QA-201: Reproducible cold boots (≥10) of identified build on target hardware
- Rule N-01: Zero external crates in TCB
- Rule N-04: Zero unsafe code in new userspace modules
- Rule N-06 / PRD-001 §7.5: Hardware verification with deterministic bounds and non-destructive recovery

---

## 1. Purpose & Scope

Sprint SP-107 enforces strict hardware boot reproducibility on the Motorola Moto G60 (`hanoip`, Qualcomm SM7150 Snapdragon 732G). Per Table 19.2 of the Ind OS Development Bible, Gate G9 requires an empirical demonstration of at least **10 consecutive reproducible cold boots** without boot hangs, watchdog timeouts, memory corruptions, or peripheral bus stalls.

This specification details:
1. Power-on reset (POR) verification and clean DRAM initialization.
2. Hardware register inspection for reset reason discrimination (Cold POR vs Warm Reboot vs Panic Reset).
3. Deterministic boot timing analysis across 10 consecutive cold boots (enforcing timing jitter $\Delta t < 50\text{ ms}$).
4. Image integrity CRC32/SHA verification across cycles to guarantee immutable execution paths.

---

## 2. Invariants

| ID | Description | Enforcement Mechanism |
|---|---|---|
| **INV-BOOT-COLD-001** | **Deterministic Power-On-Reset** | Hardware reset register must read `0x00000001` (POR), proving a cold power cycle and guaranteed DRAM retention state clear. |
| **INV-BOOT-COLD-002** | **Bounded Boot Jitter Across ≥10 Cycles** | Across 10 cold boots, boot latency must remain within $820 \pm 50\text{ ms}$; zero timeouts, zero bus stalls, and zero panic triggers allowed. |
| **INV-BOOT-COLD-003** | **Binary Fingerprint & Reproducibility Consistency** | Kernel text and rodata checksum (`0x5C01D92E`) must evaluate identically across all 10 cold runs, confirming deterministic binary execution. |

---

## 3. Hardware Architecture & Reset Registers

On Qualcomm SM7150:
- **PMIC PON (Power-On) Peripheral:** Tracks power-up triggers (Power key press, USB insertion, or Hardware Watchdog).
- **GCC (Global Clock Controller) Reset Reason Register:** Located at base `0x01400000 + 0x65000`. A cold boot from cold power state sets bit 0 (`GCC_RESET_STATUS_POR = 0x00000001`).
- **Cold Boot Cycle Protocol:**
  1. Complete PMIC power rail sequencing (VREG_L5A, VREG_L12A, VREG_S4A).
  2. Release AP CPU core reset.
  3. Enter EL2 -> EL1 transition.
  4. Kernel boots through Stage 001 to Stage 489.
  5. Measurement taken from first instruction cycle counter to userspace PID 1 init stage.

---

## 4. API & Capability Model

Operations require the specific capability token `CAP_COLD_BOOT_VERIFY` (`0x0000_1070`).

```rust
pub const CAP_COLD_BOOT_VERIFY: u32 = 0x0000_1070;
pub const MIN_COLD_BOOT_RUNS: usize = 10;
pub const MAX_BOOT_JITTER_MS: u32 = 50;
pub const EXPECTED_RESET_REASON_POR: u32 = 0x0000_0001;
pub const KERNEL_IMAGE_CRC32: u32 = 0x5C01_D92E;
pub const NOMINAL_COLD_BOOT_TIME_MS: u32 = 820;
```

---

## 5. Verification Plan

The verification script `ci/verify_sp107.py` validates:
1. Presence of all SP-107 architecture, validation, userspace, and kernel deliverables.
2. Compliance with Rule N-01 (zero external crate dependencies).
3. Compliance with Rule N-04 (zero unsafe code blocks in userspace).
4. Full execution of Stages 485–489 in golden boot logs.
5. Invariant assertion checks (INV-BOOT-COLD-001..003) confirming 10/10 cold boots succeeded with jitter < 50ms.
