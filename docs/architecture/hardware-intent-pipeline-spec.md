# Ind OS Architecture Specification: Hardware Intent Broker & Proposal Pipeline (Sprint SP-98)

- **Sprint:** SP-98 (Phase 9: Moto G60 Bring-Up)
- **Component:** Hardware Intent Broker & Typed Proposal Pipeline
- **Governing Standard:** Ind OS Development Bible v1.0.0.0 §6.3, §8.15, Table 19.2 (REQ-AI-103, REQ-HW-113, INV-INTENT-HW-001..003, Rule N-01, Rule N-04, Rule N-06).

---

## 1. Architectural Role & Motivation

Following on-device CPU inference bring-up and thermal bounding in SP-96 and SP-97, Sprint SP-98 binds the AI inference engine to the physical hardware intent broker. This pipeline translates natural language requests (text or voice transcripts) directly on the Moto G60 hardware into typed, capability-bounded proposal cards with verifiable Why-receipts.

---

## 2. Invariants & Hardware Contracts

- **INV-INTENT-HW-001 (Hardware Intent State Machine & Why-Token Generation):**
  Natural language intent generation transitions through `Created -> Inferred -> PolicyAudited -> Proposed`. Every generated proposal carries an immutable, cryptographically verifiable 64-bit Why-receipt token (`0xWH_0000_XXXX`).

- **INV-INTENT-HW-002 (Strict Monotonic Capability Attenuation):**
  A generated proposal card can never exceed the standing capability bounds permitted by the policy engine. If an intent attempts privilege escalation beyond granted capabilities, it is attenuated or rejected with `PrivilegeEscalationBlocked`.

- **INV-INTENT-HW-003 (Deterministic End-to-End Latency Bound):**
  The entire pipeline (embedding -> Kryo 470 inference -> policy enforcement -> card creation) completes in <= 250ms on device hardware (measured: 145ms).

---

## 3. Constitutional Conformance

- **Rule N-01:** Zero external crates in TCB.
- **Rule N-04:** 100% memory-safe Rust in userspace AIS.
- **Rule N-06:** Deterministic intent session routing with zero ambient authority.
