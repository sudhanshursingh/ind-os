# Decision Record: DEC-001 — From-Scratch Kernel & Operating System

**Status:** RATIFIED & LOCKED  
**Date:** 2026-10-26  
**Decider:** Founder (Sudhanshu Raj Singh)  
**Governing Document:** `Ind_OS_Development_Bible_v1.0.0.0.md` (INDOS-BIBLE-001, Section 1.2)  
**Requirement References:** `REQ-GEN-001`, `DEC-001`, `Rule N-01`

---

## 1. Context & The Audit Divergence

In the architectural audit accompanying INDOS-PRD-002, the reviewers analyzed three paths:
- **Option A:** Android foundation (forked AOSP / LineageOS 22.x).
- **Option B:** Linux-mobile foundation (postmarketOS / mainline Linux).
- **Option C:** A genuinely new kernel and userspace written from scratch.

The audit advised *against* Option C for a first product citing severe schedule risk, driver famine on Qualcomm hardware (SDM732G), and lack of an existing application ecosystem.

## 2. Founder Decision & Explicit Override

The founder has explicitly reviewed the audit's findings and **overrides the recommendation**:

> **DEC-001 (restated):** Ind OS will use a **from-scratch kernel written from scratch** in memory-safe Rust (the path labeled Option C in the audit).

## 3. Rationale

1. **True Technological Sovereignty:** An operating system claiming to place the user in absolute control cannot inherit Google's or the Linux Foundation's architectural decisions, IPC bloat, or ambient authority.
2. **Security Below AI (Rule N-04):** Monolithic kernels and Android's `system_server` architecture cannot formally enforce that the intelligence subsystem (AIS) has zero ambient privilege. A small, capability-based microkernel core is the only mathematically verifiable path.
3. **No Legacy Bloat:** Standard mobile stacks require 4–8 GB of RAM merely to boot an idle framework. A from-scratch system can boot in megabytes, reserving hardware resources for local AI inference.

## 4. Consequences & Pre-Locked Mitigations

- **Driver Famine (Risks R-01, R-02):** The Moto G60 Qualcomm SoC requires proprietary firmware and driver support. We mitigate this by developing the first ~60% of the OS on ARM64 QEMU virt (`virt` machine), deferring physical SoC driver bring-up to Phase 9.
- **Ecosystem Strategy (PEN-013):** Phase 0 proceeds on toolchain, specifications, and test harnesses. The third-party app strategy is formally categorized under PEN-013.
- **Gate Re-Examination:** This decision is subject to re-examination at Gate G0 and Gate G3 against empirical evidence.
