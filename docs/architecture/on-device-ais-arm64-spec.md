# Ind OS Architecture Specification: On-Device AIS Runtime ARM64 Port & CPU Benchmark (Sprint SP-96)

- **Sprint:** SP-96 (Phase 9: Moto G60 Bring-Up)
- **Component:** On-Device AIS Runtime Engine & ARM64 CPU Inference Benchmark
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2 (REQ-AI-101, REQ-HW-113, INV-AIS-HW-001..003, Rule N-01, Rule N-04, Rule N-06).

---

## 1. Architectural Role & Motivation

Per Bible §8.15 and PEN-008c/009 resolutions, Ind OS requires honest on-device AI runtime execution without reliance on cloud offload. Sprint SP-96 ports the AIS inference runtime to ARM64 bare-metal/hardware execution on the Qualcomm Snapdragon 732G (Kryo 470 octa-core CPU).

---

## 2. Invariants & Hardware Contracts

- **INV-AIS-HW-001 (Isolated Sandboxed ARM64 AIS Engine):**
  The AIS runtime executes in a zero-ambient-authority memory domain (`CAP_AIS_EXEC` `0x0002_0000`), with NEON SIMD vector dispatch enabled.

- **INV-AIS-HW-002 (Kryo 470 Inference Latency Benchmark):**
  Quantized 4-bit/8-bit intent embedding & classification completes within the target latency budget of <= 120ms (measured benchmark recorded on hardware core).

- **INV-AIS-HW-003 (Strict 64MB RAM Quota Enforcement):**
  The runtime memory footprint is monitored and bounded; allocations exceeding 64MB trigger `MemoryQuotaExceeded` without compromising system memory.

---

## 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB. In-tree pure Rust inference engine.
- **Rule N-04:** 100% memory-safe Rust in userspace AIS.
- **Rule N-06:** Deterministic memory bounds with zero ambient authority.
