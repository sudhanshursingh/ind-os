# Ind OS Architecture Specification: Trusted Consent Broker Subsystem v1

**Document Identifier:** `INDOS-SPEC-CON-01`  
**Classification:** Core Kernel Architecture / Security Spine Specification  
**Status:** Approved & Implemented (Sprint SP-21)  
**Governing Standard:** Ind OS Development Bible v1.0.0.0 (§8.9, §10, §14)  
**Invariants Enforced:** `INV-CON-001`  

---

## 1. Executive Summary

The Ind OS Trusted Consent Broker (`Consent Broker v1`) establishes an un-spoofable, kernel-mediated user consent pipeline within the IndK microkernel. Under the zero-ambient-authority architecture of Phase 3, high-privilege hardware operations (e.g., camera, microphone, secure element, sensitive AIS services) cannot be accessed purely by capability tokens without standing policy and explicit user authorization.

To eliminate vulnerabilities such as overlay clickjacking, synthetic input injection, and window hijacking, the Consent Broker binds directly to a dedicated, hardware-isolated display plane and enforces kernel-exclusive input preemption whenever a security prompt is active.

---

## 2. Normative Invariants

### `INV-CON-001`: Unspoofable Consent Surface & Hardware Input Isolation
> **Invariant Statement:** When a consent evaluation is active, the consent dialog displays exclusively on a hardware-isolated display plane (`PlaneId::HardwareIsolated`) inaccessible to userspace compositors and untrusted tasks. User input (touch, key events) is exclusively routed to the kernel-mediated consent handler. Any attempt by an untrusted user process to draw on the display plane or inject synthetic input while a consent prompt is active must be deterministically rejected with `EPERM`.

---

## 3. Subsystem Architecture

The Consent Broker architecture is organized into three kernel components:

```
+-------------------------------------------------------------------+
|                        IndK Microkernel                           |
|                                                                   |
|   +---------------------+        +----------------------------+   |
|   |  Policy Engine v1   |        |   Cryptographic Audit Log  |   |
|   | (NeedsConsent rule) |        |       (INV-AUD-001)        |   |
|   +----------+----------+        +-------------+--------------+   |
|              |                                 ^                  |
|              v                                 | (Audit Linkage)  |
|   +--------------------------------------------+--------------+   |
|   |             Consent Broker (ConsentBroker)                |   |
|   |  - request_consent()                                      |   |
|   |  - submit_decision()                                      |   |
|   +---------------------+-------------------------------------+   |
|                         |                                         |
|                         v                                         |
|   +-----------------------------------------------------------+   |
|   |             IsolatedSurface (INV-CON-001)                 |   |
|   |  - Hardware Display Plane: PlaneId::HardwareIsolated      |   |
|   |  - Hardware Input Preemption: input_exclusive = true      |   |
|   |  - Overlay / Injection Defense: 100% EPERM Rejection      |   |
|   +-----------------------------------------------------------+   |
+-------------------------------------------------------------------+
```

### 3.1 Components
1. **[`IsolatedSurface`](file:///Users/dhan/IndOS/kernel/src/consent/surface.rs):** Manages hardware-isolated display planes and input preemption.
2. **[`ConsentRecord`](file:///Users/dhan/IndOS/kernel/src/consent/record.rs):** Bounded data structure recording request metadata, rights requested, granted rights, decision, and audit sequence numbers.
3. **[`ConsentBroker`](file:///Users/dhan/IndOS/kernel/src/consent/broker.rs):** State machine managing the lifecycle of consent prompts and audit chaining.

---

## 4. Consent Record Schema

```rust
pub enum ConsentDecision {
    Pending = 0,
    Granted = 1,
    Denied = 2,
    TimedOut = 3,
}

pub struct ConsentRecord {
    pub consent_id: u64,
    pub requesting_pid: ProcessId,
    pub resource_id: u32,
    pub requested_rights: CapRights,
    pub granted_rights: CapRights,
    pub decision: ConsentDecision,
    pub timestamp: u64,
    pub audit_seq: u64,
}
```

---

## 5. Defense Against Overlay & Input Injection (`INV-CON-001`)

During an active prompt:
1. **Overlay Rejection:** Any process with `caller_pid != 0` attempting to modify the surface or allocate framebuffers is rejected with `Err("EPERM: Access denied to isolated consent display surface (INV-CON-001)")`.
2. **Synthetic Input Rejection:** Any process with `caller_pid != 0` attempting to deliver synthetic touch or key events is rejected with `Err("EPERM: Synthetic input injection rejected during active consent prompt (INV-CON-001)")`.
3. **Audit Proof:** All consent lifecycle transitions are permanently recorded in the SHA-256 hash-chained audit log with sequence tags.

---

## 6. Verification & Test Evidence

Validated in QEMU via boot stages 53 through 56:
- `Stage 53 (53_CONSENT_SURFACE_INIT)`: Validates initialization of inactive plane.
- `Stage 54 (54_CONSENT_REQUEST_EVAL)`: Executes full request, prompt activation, user grant, and audit linkage.
- `Stage 55 (55_CONSENT_SPOOF_INJECTION_DEFENSE)`: Demonstrates 100% deterministic rejection of adversarial overlay and synthetic input injection (`INV-CON-001 PASSED`).
- `Stage 56 (56_CONSENT_BROKER_READY)`: Milestone completion verification.
