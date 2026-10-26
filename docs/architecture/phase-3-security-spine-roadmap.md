# Phase 3 Strategic Architecture & Execution Roadmap: The Security Spine

**Document ID:** `INDOS-STRAT-PHASE3-01`  
**Governing Standard:** `Ind_OS_Development_Bible_v1.0.0.0.md` (§8.9, §10, §14, §19.2)  
**Phase:** Phase 3 — The Security Spine: IPC + Capabilities + Audit (Sprints SP-16 through SP-23) $\rightarrow$ **Gate G3 (HARD STOP)**  
**Target Milestone:** Live Enforcement of `REQ-KRN-104` (Capability Spine), `REQ-KRN-105` (Zero Ambient Authority), `REQ-KRN-108` (Microkernel IPC Rendezvous), and `REQ-SEC-106` (Monotonic Attenuation & Revocation Soundness)  
**Date:** 2026-10-26  
**Status:** RATIFIED & ACTIVE  

---

## 1. Executive Summary & Phase Charter

Phase 3 is the foundational architecture upon which all subsequent layers of Ind OS (VFS, drivers, networking, windowing, and AIS runtime) strictly rely. In accordance with Development Bible §8.9:
> *"This is the phase the whole OS stands on; it gets two extra sprints and the strictest gate in the programme."*

The core mission of Phase 3 is to transition Ind OS from a preemptive, address-space-isolated microkernel into a **pure capability-based capability-system operating system** governed by:
* **Rule N-04 (Security is enforced, never requested):** Privilege is never granted via ambient root roles or implicit ambient authority.
* **Rule N-05 (Zero ambient authority):** A process possesses strictly zero authority to invoke any kernel service, endpoint, memory region, or hardware resource unless presenting an unforgeable, capability-bearing token.

---

## 2. Strategic Watchpoints & Architectural Mitigations

In alignment with technical leadership recommendations and architectural review, Phase 3 establishes four mandatory engineering guardrails:

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        PHASE 3 STRATEGIC ARCHITECTURAL GUARDRAILS                      │
├──────────────────────────┬─────────────────────────────────────────────────────────────┤
│ 1. IPC Latency Control   │ Inline fast-path capability validation caching (< 10 μs)    │
│ 2. Revocation Soundness  │ Intrusive dependency graphs avoiding recursion lock spikes  │
│ 3. Consent UI Isolation  │ Dedicated kernel-mediated display plane & secure input      │
│ 4. Gate G3 Hard Stop     │ Continuous adversarial red-teaming (SP-22) before audit     │
└──────────────────────────┴─────────────────────────────────────────────────────────────┘
```

### 2.1 Watchpoint 1: IPC Latency Overhead Control (SP-16 & Benchmarking)
* **The Challenge:** Introducing rich endpoints, rights checking, capability badge verification, notifications, and timeouts on every IPC interaction risks inflating the $2.4\ \mu\text{s}$ median context-switch baseline established at Gate G2.
* **Architectural Mitigation:**
  1. Implement **inline capability-lookup fast-path caching** inside the per-process capability namespace.
  2. Maintain register-mediated synchronous rendezvous paths (`x0..x7`) without heap allocation or intermediate buffer copies.
  3. Ensure round-trip IPC latency across user processes remains strictly bounded under **$< 10\ \mu\text{s}$** (measuring and logging this in the SP-16 benchmark harness as a permanent G4 denominator).

### 2.2 Watchpoint 2: Capability Attenuation & Revocation Propagation (SP-17 & SP-18)
* **The Challenge:** Cascading recursive revocation across multi-level capability delegation trees can trigger unbounded traversal latencies and severe lock contention across process tables during mass revocation.
* **Architectural Mitigation:**
  1. Structure capability lineage using **intrusive doubly-linked tree references** (`parent_slot`, `first_child`, `next_sibling`) within pre-allocated bounded tables.
  2. Implement bounded, non-recursive worklist traversal for mass revocation events.
  3. Ensure mathematical monotonicity ($\forall R_{\text{child}}, R_{\text{parent}} : (R_{\text{child}} \ \& \ \sim R_{\text{parent}}) = 0$) is enforced at minting time, making attenuation verification $O(1)$.

### 2.3 Watchpoint 3: Consent Broker Surface Isolation (SP-21 & Gate G3)
* **The Challenge:** Gate G3 mandates that the **Consent Broker UI must be provably un-spoofable** by untrusted user processes (immune to clickjacking, malicious overlays, and fake input injection).
* **Architectural Mitigation:**
  1. Ensure the Consent Broker communicates over a **dedicated, kernel-mediated display plane** that bypasses standard userspace compositing and damage buffers.
  2. Route user touch/keyboard input through a direct kernel-isolated hardware channel that preempts untrusted display surfaces when a security prompt is active.
  3. Establish an un-spoofable cryptographic consent record schema verified by the append-only audit log.

### 2.4 Watchpoint 4: Gate G3 "Hard Stop" & Early Adversarial Red-Teaming (SP-22 & SP-23)
* **The Challenge:** Gate G3 is classified as a **HARD STOP** in the Development Bible (§19.2). Failure triggers a complete program halt and whole-architecture re-review by the founder. Phase 4 will not begin on a compromised security boundary.
* **Architectural Mitigation:**
  1. Integrate adversarial fuzzing and red-teaming directly into the development cycle (SP-22), actively probing for capability aliasing, badge confusion, and audit log tampering.
  2. Require machine-checkable formal specification groundwork (`INV-CAP-001`, `INV-CAP-002`, `INV-IPC-001`, `INV-AUD-001`) before assembling the final Gate G3 evidence pack in SP-23.

---

## 3. Sprint-by-Sprint Execution Roadmap (SP-16 through SP-23)

| Sprint | Subsystem & Scope | Key Invariants & Deliverables | Gate Dependency |
| :--- | :--- | :--- | :---: |
| **SP-16** | **IPC Core:** Synchronous & asynchronous endpoints, rights transfer, notifications, deadlines/timeouts, latency benchmark harness. | `INV-IPC-001` (Rendezvous), `INV-IPC-002` (Bounded Latency $< 10\ \mu\text{s}$), `docs/evidence/ipc-latency-baseline.md`. | G4 input |
| **SP-17** | **Capabilities I:** Unforgeable tokens, kernel object-table integration, mint/grant/revoke, per-process namespaces. | `INV-CAP-001` (Monotonic Attenuation), `INV-OBJ-001` (Generational Anti-ABA), per-process `CapTable`. | Core spine |
| **SP-18** | **Capabilities II:** Delegation with attenuation, recursive revocation propagation, capability-leak test suite. | `INV-CAP-002` (Revocation Soundness), zero-leakage invariant, transitive revocation tests. | Core spine |
| **SP-19** | **Policy Engine v1 (ENF):** Declarative, human-readable security policy language, default-deny rule evaluation, versioned rule artifacts. | `INV-ENF-001` (Default Deny), `INV-ENF-002` (Deterministic Rule Resolution). | Core spine |
| **SP-20** | **Audit v1:** Hash-chained append-only cryptographic security log, log signing, tamper-detection verification suite. | `INV-AUD-001` (Append-Only Hash Chain), bit-flip tamper detection test harness. | Gate G3 |
| **SP-21** | **Consent Broker v1:** Trusted UI channel end-to-end, isolated display surface, kernel-mediated input, consent record schema. | `INV-CON-001` (Unspoofable Consent Surface), input-injection rejection test. | Gate G3 |
| **SP-22** | **Adversarial Self-Review:** Internal red-team exploitation attempts (privilege escalation, capability confusion, audit tampering). | 100% resolution or founder acceptance of all findings, adversarial report. | Gate G3 |
| **SP-23** | **Formal-Spec Groundwork & Gate G3 Evidence Assembly:** Machine-checkable specs, zero-ambient authority proof (grep + runtime). | `GATE-G3-AUDIT.md`, `evidence/G3/manifest.json`, DEC-001/DEF-001 reviews. | **HARD STOP** |

---

## 4. Phase 3 Invariant Ledger

1. **`INV-IPC-001` (Zero-Copy Synchronous Rendezvous):** Message exchange occurs directly across hardware registers without intermediate kernel heap buffering.
2. **`INV-IPC-002` (Deterministic IPC Round-Trip Latency):** Microkernel rendezvous message passing latency must not exceed $10.0\ \mu\text{s}$ under uncontended execution.
3. **`INV-CAP-001` (Monotonic Rights Attenuation):** Authority derived from a capability must strictly be a subset of the parent capability's rights ($\forall R_{\text{child}}, R_{\text{parent}} : (R_{\text{child}} \ \& \ \sim R_{\text{parent}}) = 0$).
4. **`INV-CAP-002` (Revocation Soundness):** Revoking a capability transitively and deterministically invalidates all derived descendant capabilities across all process namespaces.
5. **`INV-SEC-001` (Zero Ambient Authority):** No kernel system call or resource invocation succeeds without presentation of a valid, typed capability handle.
6. **`INV-AUD-001` (Cryptographic Append-Only Audit Integrity):** Audit records form a SHA-256 hash chain ($H_n = \text{SHA256}(H_{n-1} \parallel R_n)$); any historical bit flip breaks the chain verification.
7. **`INV-CON-001` (Unspoofable Consent Display):** The consent dialog displays exclusively on a hardware-isolated plane inaccessible to user-space compositors.

---

## 5. Alignment with Programme Governance & Deviation Policy

* **Deviation Status:** **ZERO DEVIATIONS.**
* **Rule N-01:** Pure `no_std` Rust + AArch64 assembly, zero external kernel crates.
* **Rule N-04 / N-05:** Capabilities are mandatory tokens, not optional advisory ACLs.
* **Rule N-10:** Conventional commits with signed-off-by (`git commit --signoff`).
* **Rule N-13:** Preservation of all completed sprint contracts (SP-00 through SP-15).
