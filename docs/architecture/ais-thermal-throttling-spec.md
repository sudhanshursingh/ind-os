# Ind OS Architecture Specification: AIS Memory Bounds & Thermal Throttling (Sprint SP-97)

- **Sprint:** SP-97 (Phase 9: Moto G60 Bring-Up)
- **Component:** AIS Dynamic Thermal Throttling Governor & Memory Bounds
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2 (REQ-AI-102, REQ-THM-102, INV-AIS-THM-001..003, Rule N-01, Rule N-04, Rule N-06).

---

## 1. Architectural Role & Motivation

Continuous on-device neural model inference on a physical smartphone SoC can induce severe thermal dissipation if unthrottled. Sprint SP-97 integrates the userspace AIS inference pipeline with the hardware Thermal Management Subsystem (brought up in SP-82) to govern inference duty cycles dynamically.

---

## 2. Invariants & Hardware Contracts

- **INV-AIS-THM-001 (Thermal Throttle Threshold at 42.0°C):**
  When SoC thermal zone reports temperature >= 42,000 mC, the AIS governor reduces inference concurrency from 100% to 50% duty-cycle and rejects low-priority proactive embeddings.

- **INV-AIS-THM-002 (Emergency Thermal Halting at 46.0°C):**
  If temperature reaches or exceeds 46,000 mC, all background AIS model inference is paused immediately (`ThermalHalt`), allowing only interactive user-initiated intents.

- **INV-AIS-THM-003 (Deterministic Cooling Recovery & Hysteresis):**
  Inference resumes full throughput only when temperature falls below 39,000 mC (3.0°C hysteresis band), preventing rapid oscillation.

---

## 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** 100% memory-safe Rust in `userspace/ais`.
- **Rule N-06:** Deterministic thermal loop bounds with zero dynamic allocation.
