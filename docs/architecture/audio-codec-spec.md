# Ind OS Architecture Specification: Audio Codec Bring-Up (Sprint SP-93)

- **Sprint:** SP-93 (Phase 9: Moto G60 Bring-Up)
- **Component:** Audio Codec (Qualcomm WCD9385 / SLIMbus / SoundWire)
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2 (REQ-HW-111, REQ-AUD-101, INV-AUD-001..003, Rule N-01, Rule N-04, Rule N-06).

---

## 1. Architectural Role & Motivation

Per Bible §8.15, mobile device bring-up requires audio codec hardware enablement. The Motorola Moto G60 (`hanoip`) utilizes a Qualcomm WCD9385 high-definition audio codec interfaced via Qualcomm SLIMbus / SoundWire audio digital interface.

Sprint SP-93 delivers the foundational hardware abstraction layer and device manager driver for audio codec discovery, register initialization, and analog power-sequencing.

---

## 2. Invariants & Hardware Contracts

- **INV-AUD-001 (Audio Controller MMIO Mapping):**
  The SLIMbus / SoundWire audio controller is mapped at physical base `0x0c00_0000` with 64KB aperture. Mapping is capability-gated by `CAP_DEVICE_ADMIN` (`0x0000_0004`). Unprivileged mapping attempts fail with `PermissionDenied`.

- **INV-AUD-002 (WCD9385 Identification & Register Handshake):**
  Reading codec identification register returns manufacturer ID `0x42` and product ID `0x85` (WCD9385). Non-matching identifiers abort with `CodecIdentificationFailed`.

- **INV-AUD-003 (Analog Charge Pump & Power-Up Sequencing):**
  Enabling the codec executes an orderly power-up sequence: Class-H charge pump enabled -> PLL locked -> DAC/ADC digital power rails stabilized.

---

## 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** 100% memory-safe Rust in `userspace/devmgr`.
- **Rule N-06:** Deterministic register access with bounded retry counts.
