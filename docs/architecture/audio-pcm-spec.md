# Ind OS Architecture Specification: Audio PCM Stream Buffer & User-Mode HAL (Sprint SP-94)

- **Sprint:** SP-94 (Phase 9: Moto G60 Bring-Up)
- **Component:** Audio PCM Stream Engine & DMA Buffer Manager
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2 (REQ-HW-111, REQ-AUD-102, INV-PCM-001..003, Rule N-01, Rule N-04, Rule N-06).

---

## 1. Architectural Role & Motivation

Following WCD9385 codec discovery in SP-93, Sprint SP-94 establishes the PCM digital audio streaming pipeline and user-mode audio HAL driver. This component enables digital audio playback (and recording) across the Moto G60 hardware abstraction layer without unbounded memory allocation or jitter.

---

## 2. Invariants & Hardware Contracts

- **INV-PCM-001 (Capability-Gated PCM Ring-Buffer Allocation):**
  Audio PCM ring-buffer allocation and configuration (48 kHz sample rate, 16-bit signed PCM stereo, 4 periods) is gated by `CAP_AUDIO_PLAYBACK` (`0x0000_0008`). Unprivileged calls return `PermissionDenied`.

- **INV-PCM-002 (XRUN Resilience & Clean Zero-Padding):**
  If userspace fails to supply audio frames before DMA hardware pointer advances past the buffer boundary, the hardware driver detects the underrun, increments an XRUN counter, and injects zero-amplitude silence samples rather than hanging or panicking.

- **INV-PCM-003 (Bounded Latency Pipeline):**
  Buffer periods are sized for <= 20ms latency at 48kHz stereo (e.g. 192 samples/period = 4ms per period), satisfying real-time multimedia deadlines.

---

## 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** 100% memory-safe Rust in `userspace/devmgr`.
- **Rule N-06:** Deterministic circular buffering with zero dynamic heap allocation in playback path.
