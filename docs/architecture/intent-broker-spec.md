# Ind OS Architecture Specification: Intent Broker & Typed Proposal Schema

**Document ID:** `INDOS-SPEC-INTENT-01`  
**Sprint:** SP-42 (Intent Broker: NL/System-Event -> Typed Proposal Schema, Schema Registry, Pre-Routing Logging)  
**Phase:** Phase 6 (AIS Subsystem + ENF Policy Engine + Consent Broker)  
**Constitutional Authority:** Ind OS Development Bible v1.0.0.0 §6.3, §8.12 (REQ-AI-003, REQ-AI-102)  
**Status:** **RATIFIED**  
**Classification:** Core System Architecture Specification  

---

## 1. Scope & Objective

In Ind OS, the AI Subsystem (AIS) is **strictly advisory** (Bible §6.3, REQ-AI-001, REQ-AI-003). It never executes system actions directly. Instead, any natural language prompt or system event is translated by the **Intent Broker** into a **Typed Proposal**.

Sprint SP-42 implements the Intent Broker and Schema Registry:
1. **Typed Proposal Generation (`REQ-AI-102`):** Converts unstructured user text or hardware events into strongly-typed, bounded proposal records conforming to registered schemas.
2. **Schema Registry (`REQ-AI-102`):** Maintains the canonical catalog of valid system actions (e.g., `DeviceAction`, `FileAccess`, `SystemSetting`, `Notification`, `NetworkTransfer`). Unregistered or malformed schemas are rejected deterministically.
3. **Pre-Routing Audit Logging (`REQ-AI-003`):** Every proposal MUST be immutably recorded to the structured log bus (`logd`) *before* being routed to the Execution Normalization Framework (ENF), policy engine, or consent broker. Denied or invalid proposals are logged as well.
4. **Zero Ambient Authority (`REQ-AI-001`):** A proposal object carries zero execution privileges; it is solely an advisory intent awaiting ENF policy evaluation and user consent.

---

## 2. Invariants & Architectural Rules

### 2.1 Invariant `INV-INTENT-001`: Typed Proposal Schema Enforcement
Every proposal emitted by the Intent Broker MUST conform to a registered schema in the `SchemaRegistry`:
- Actions must possess valid category identifiers, parameter counts, and data boundaries.
- Schemas cannot contain executable bytecode or raw pointer representations.
- Proposals failing schema validation are dropped with `InvalidSchema`.

### 2.2 Invariant `INV-INTENT-002`: Pre-Routing Proposal Audit Logging (REQ-AI-003)
Under REQ-AI-003, no proposal can be dispatched to downstream consumers (Policy Engine or Consent Broker) without first emitting an immutable audit event:
- Event contains: `proposal_id`, `source_type` (NL vs SystemEvent), `action_type`, `confidence_score`, and `hash`.
- Ensures a complete historical record of what AIS contemplated doing, regardless of whether it was later denied, modified, or approved.

### 2.3 Invariant `INV-INTENT-003`: Advisory-Only Intent Posture
A generated `TypedProposal` has zero ambient authority:
- Does NOT execute actions or invoke capabilities directly.
- Encapsulates target capability requirements (`required_rights`) that ENF must verify.
- Unmediated attempts by AIS to execute proposals without policy approval fail with `EPERM`.

---

## 3. Data Structures & Subsystem Architecture

### 3.1 Proposal Action Category
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionCategory {
    DeviceControl,   // e.g. toggle bluetooth/wifi
    FileOperation,   // e.g. read/write document
    SystemSetting,   // e.g. brightness, volume
    Notification,    // e.g. alert user
    NetworkTransfer, // e.g. sync backup
}
```

### 3.2 Typed Proposal Representation
```rust
#[derive(Debug, Clone, Copy)]
pub struct TypedProposal {
    pub proposal_id: u32,
    pub category: ActionCategory,
    pub action_code: u32,
    pub target_id: u32,
    pub confidence_score: u8, // 0..100
    pub required_rights: u32,
    pub pre_logged: bool,
}
```

---

## 4. Verification & Acceptance Protocol

The implementation of SP-42 is validated through:
1. Subsystem implementation within [`userspace/ais/`](file:///Users/dhan/IndOS/userspace/ais/) (`schema.rs`, `intent.rs`).
2. Kernel boot verification stages 157–161 in [`kernel/src/main.rs`](file:///Users/dhan/IndOS/kernel/src/main.rs).
3. QEMU execution log [`docs/evidence/golden-boot-sp42.log`](file:///Users/dhan/IndOS/docs/evidence/golden-boot-sp42.log).
4. Validation report [`docs/evidence/SP-42-validation.md`](file:///Users/dhan/IndOS/docs/evidence/SP-42-validation.md).
5. Automated CI verifier [`ci/verify_sp42.py`](file:///Users/dhan/IndOS/ci/verify_sp42.py).
