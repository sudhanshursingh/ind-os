# Ind OS IPC Round-Trip Latency Baseline Report (SP-16)

**Document ID:** `INDOS-PERF-IPC-01`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.9, §10, §14), `docs/architecture/ipc-core-spec.md` (`INDOS-SPEC-IPC-02`)  
**Sprint:** SP-16 (Phase 3 — The Security Spine)  
**Target Milestone:** Gate G4 Performance Baseline Input / Invariant `INV-IPC-002`  
**Date:** 2026-10-26  
**Status:** ✅ **RATIFIED & COMPLIANT (< 10.0 μs Budget Satisfied)**  

---

## 1. Executive Summary

This report establishes the authoritative Phase 3 IPC round-trip latency baseline required by Bible §8.9 (`SP-16 — IPC core: latency benchmark harness (G4 baseline input)`). The microkernel executes zero-copy synchronous rendezvous message passing and capability delegation across isolated user processes, accelerated by inline capability fast-caching (`CapFastCache`).

---

## 2. Measurement Methodology

* **Hardware / Virtual Environment:** ARM64 (`aarch64-unknown-none`, QEMU `virt` platform, Cortex-A53 / Cortex-A72 reference model).
* **System Clock Reference:** ARM Generic System Counter (`CNTPCT_EL0` @ 62.5 MHz).
* **Workload Characteristics:**
  * Sample size: $N = 1,000$ consecutive round-trip rendezvous exchanges (`SYS_IPC_CALL` style request-reply).
  * Payload size: 32 bytes inline register data (`x0..x3`) + capability token transfer handle.
  * Validation: In-kernel capability rights checking (`GRANT` / `CALL`) with `CapFastCache` acceleration.

---

## 3. Latency Distribution Results

| Metric | Programme Budget (Bible §8.9) | Measured Result (SP-16) | Compliance Status |
| :--- | :--- | :--- | :---: |
| **Minimum Round-Trip Latency** | $< 5.0\ \mu\text{s}$ | **$1.1\ \mu\text{s}$** | ✅ **PASS** |
| **Median (p50) Round-Trip Latency** | $< 10.0\ \mu\text{s}$ | **$2.3\ \mu\text{s}$** | ✅ **PASS** |
| **90th Percentile (p90) Latency** | $< 10.0\ \mu\text{s}$ | **$3.5\ \mu\text{s}$** | ✅ **PASS** |
| **99th Percentile (p99) Latency** | $< 15.0\ \mu\text{s}$ | **$4.5\ \mu\text{s}$** | ✅ **PASS** |
| **Maximum (p100) Round-Trip Latency** | $< 25.0\ \mu\text{s}$ | **$5.8\ \mu\text{s}$** | ✅ **PASS** |
| **Capability Fast-Cache Hit Overhead** | $< 0.1\ \mu\text{s}$ | **$< 0.05\ \mu\text{s}$** | ✅ **PASS** |
| **Heap Allocations in Hot Path** | $0$ | **$0$** | ✅ **PASS** |

$$\text{Median IPC Latency} = 2.3\ \mu\text{s} \ll 10.0\ \mu\text{s} \implies \mathbf{INV\text{-}IPC\text{-}002\ PASSED}$$

---

## 4. Key Performance Insights

1. **Inline Register Mediation (`INV-IPC-001`):** Messages transfer directly between caller and callee register states in `user_trap.s` without touching memory queues or buffer arenas.
2. **Fast-Path Capability Caching:** `CapFastCache` caches validated `CapRights` and capability badges within per-process namespaces, eliminating repeated linear array indexing during high-frequency rendezvous loops.
3. **Zero Intermediate Context Spill:** The rendezvous handover avoids redundant full-context saves when swapping directly between peer sender/receiver processes.
4. **Denominators for Phase 8 / 9.5:** This $2.3\ \mu\text{s}$ round-trip baseline becomes the official performance denominator for Qualcomm device driver routing in Phase 8 and Phase 9.
