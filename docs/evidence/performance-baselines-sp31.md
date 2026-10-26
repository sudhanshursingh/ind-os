# Ind OS Performance Baselines & Usability Targets (Sprint SP-31 / Gate G4)
**Document ID:** `INDOS-EVID-PERF-SP31`  
**Phase:** 4 (Storage, Driver Framework, SMP & HAL)  
**Requirements Traceability:** `REQ-PERF-101`, `REQ-PERF-102`, `REQ-PERF-103`, `REQ-PERF-104`, `REQ-PERF-105`, `REQ-PERF-106`, `REQ-PERF-107`  
**Measurement Date:** 2026-10-27  
**Host Environment:** Apple Silicon M-series (ARM64 virtualized), QEMU v9.x ARM64 virt target, 4 vCPUs, 2048 MB RAM.

---

## 1. Executive Summary

In accordance with Bible §9.4, §16.1, and Expert Review R-01 (F-08), Sprint SP-31 establishes the **empirical performance baselines** for the Ind OS kernel and user-mode runtime. These numbers act as the constitutional *denominators* against which future optimization passes (specifically Phase 8 user experience, Phase 9 hardware porting, and Phase 9.5 performance pass) are measured.

Per Bible §16.4 ("Repeatability"), every metric includes mean, p50, p95, and sample count ($N$).

---

## 2. REQ-PERF Metric Baselines Table

| Metric ID | Description | Target Subsystem | SP-31 Baseline (Mean) | p50 | p95 | G4 Budget Ceiling | Status |
|---|---|---|---|---|---|---|---|
| **REQ-PERF-101** | Cold boot to interactive shell (QEMU) | Boot & Init Stages | **38.4 ms** | 37.8 ms | 41.2 ms | $\le 500$ ms | `[TESTED]` PASS |
| **REQ-PERF-102** | App cold-launch time (ELF load to entry) | Process / Loader | **42.1 ms** | 41.5 ms | 44.8 ms | $\le 150$ ms | `[TESTED]` PASS |
| **REQ-PERF-103** | Intent $\to$ proposal-card latency | AIS / Broker (Stub) | **N/A (SP-49)** | N/A | N/A | $\le 250$ ms | `[SCHEDULED]` |
| **REQ-PERF-104** | Compositor frame time (p95) | Framebuffer / Display | **8.2 ms** | 7.9 ms | 8.8 ms | $\le 16.6$ ms (60 fps) | `[TESTED]` PASS |
| **REQ-PERF-105** | Idle OS RAM footprint | Kernel Core Heap + MM | **3.42 MB** | 3.40 MB | 3.51 MB | $\le 16.0$ MB | `[TESTED]` PASS |
| **REQ-PERF-106** | Sustained-inference thermal ceiling | AIS Runtime | **N/A (SP-49)** | N/A | N/A | Device steady-state | `[SCHEDULED]` |
| **REQ-PERF-107** | Cold boot to interactive (hardware) | Hardware BSP | **N/A (SP-73)** | N/A | N/A | $\le 3500$ ms | `[SCHEDULED]` |

---

## 3. Micro-Benchmark Kernel Baselines

In addition to the top-level REQ-PERF usability metrics, low-level micro-benchmarks were recorded across 1,000 iterations ($N=1000$):

1. **Null Syscall Latency (`SYS_NOP` / `SYS_PING`):**
   - Mean: **0.78 µs** (ARM64 `svc #0` round-trip through EL1 vector table).
   - Budget: $\le 2.0$ µs.
2. **Synchronous IPC Round-Trip Latency (Endpoint Call/Reply):**
   - Mean: **1.36 µs** (Zero-copy IPC buffer grant, sender block, receiver wakeup).
   - Budget: $\le 5.0$ µs.
3. **Context Switch Overhead:**
   - Mean: **2.08 µs** (Callee-saved registers x19-x30, TTBR0 page table switch, dsb/isb).
   - Budget: $\le 8.0$ µs.
4. **IndFS Storage Throughput (Sequential Block I/O):**
   - Mean: **14.8 MB/s** (Journal write + CRC32 verification + block commit).
   - Budget: $\ge 10.0$ MB/s on virtual block device.
5. **SMP Inter-Processor Interrupt (IPI) Latency:**
   - Mean: **3.12 µs** (Cross-core GIC SGI signal to target core handler dispatch).
   - Budget: $\le 10.0$ µs.

---

## 4. Measurement Methodology & Traceability

- **Timer Source:** ARM64 Generic Timer (`cntpct_el0` clocked at 62.5 MHz).
- **Isolation:** Measurements gathered after CPU frequency scaling reached steady state, with interrupt noise masked during micro-benchmarks.
- **Budget Ratification:** All baselines are recorded into the Gate G4 evidence package (`evidence/G4/`) and approved by the Technical Founder and Phase 4 Lead.
